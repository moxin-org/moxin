use chrono::{DateTime, Utc};
use git2::{ProxyOptions, Repository};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::str;
use std::sync::Arc;

fn do_fetch<'a>(
    repo: &'a git2::Repository,
    refs: &[&str],
    remote: &'a mut git2::Remote,
) -> Result<git2::AnnotatedCommit<'a>, git2::Error> {
    let mut cb = git2::RemoteCallbacks::new();

    // Print out our transfer progress.
    cb.transfer_progress(|stats| {
        if stats.received_objects() == stats.total_objects() {
            log::debug!(
                "Resolving deltas {}/{}",
                stats.indexed_deltas(),
                stats.total_deltas()
            );
        } else if stats.total_objects() > 0 {
            log::debug!(
                "Received {}/{} objects ({}) in {} bytes",
                stats.received_objects(),
                stats.total_objects(),
                stats.indexed_objects(),
                stats.received_bytes()
            );
        }
        io::stdout().flush().unwrap();
        true
    });

    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(cb);
    if let Ok(proxy) = std::env::var("https_proxy").or_else(|_| std::env::var("all_proxy")) {
        let mut proxy_opt = ProxyOptions::new();
        proxy_opt.url(&proxy);
        fo.proxy_options(proxy_opt);
    }
    // Always fetch all tags.
    // Perform a download and also update tips
    // fo.download_tags(git2::AutotagOption::All);
    log::debug!("Fetching {} for repo", remote.name().unwrap());
    remote.fetch(refs, Some(&mut fo), None)?;

    // If there are local objects (we got a thin pack), then tell the user
    // how many objects we saved from having to cross the network.
    let stats = remote.stats();
    if stats.local_objects() > 0 {
        log::debug!(
            "Received {}/{} objects in {} bytes (used {} local \
             objects)",
            stats.indexed_objects(),
            stats.total_objects(),
            stats.received_bytes(),
            stats.local_objects()
        );
    } else {
        log::debug!(
            "Received {}/{} objects in {} bytes",
            stats.indexed_objects(),
            stats.total_objects(),
            stats.received_bytes()
        );
    }

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    Ok(repo.reference_to_annotated_commit(&fetch_head)?)
}

fn fast_forward(
    repo: &Repository,
    lb: &mut git2::Reference,
    rc: &git2::AnnotatedCommit,
) -> Result<(), git2::Error> {
    let name = match lb.name() {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(lb.name_bytes()).to_string(),
    };
    let msg = format!("Fast-Forward: Setting {} to id: {}", name, rc.id());
    log::debug!("{}", msg);
    lb.set_target(rc.id(), &msg)?;
    repo.set_head(&name)?;
    repo.checkout_head(Some(
        git2::build::CheckoutBuilder::default()
            // For some reason the force is required to make the working directory actually get updated
            // I suspect we should be adding some logic to handle dirty working directory states
            // but this is just an example so maybe not.
            .force(),
    ))?;
    Ok(())
}

fn normal_merge(
    repo: &Repository,
    local: &git2::AnnotatedCommit,
    remote: &git2::AnnotatedCommit,
) -> Result<(), git2::Error> {
    let local_tree = repo.find_commit(local.id())?.tree()?;
    let remote_tree = repo.find_commit(remote.id())?.tree()?;
    let ancestor = repo
        .find_commit(repo.merge_base(local.id(), remote.id())?)?
        .tree()?;
    let mut idx = repo.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;

    if idx.has_conflicts() {
        log::debug!("Merge conflicts detected...");
        repo.checkout_index(Some(&mut idx), None)?;
        return Ok(());
    }
    let result_tree = repo.find_tree(idx.write_tree_to(repo)?)?;
    // now create the merge commit
    let msg = format!("Merge: {} into {}", remote.id(), local.id());
    let sig = repo.signature()?;
    let local_commit = repo.find_commit(local.id())?;
    let remote_commit = repo.find_commit(remote.id())?;
    // Do our merge commit and set current branch head to that commit.
    let _merge_commit = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &msg,
        &result_tree,
        &[&local_commit, &remote_commit],
    )?;
    // Set working tree to match head.
    repo.checkout_head(None)?;
    Ok(())
}

fn do_merge<'a>(
    repo: &'a Repository,
    remote_branch: &str,
    fetch_commit: git2::AnnotatedCommit<'a>,
) -> Result<(), git2::Error> {
    // 1. do a merge analysis
    let analysis = repo.merge_analysis(&[&fetch_commit])?;

    // 2. Do the appropriate merge
    if analysis.0.is_fast_forward() {
        log::debug!("Doing a fast forward");
        // do a fast forward
        let refname = format!("refs/heads/{}", remote_branch);
        match repo.find_reference(&refname) {
            Ok(mut r) => {
                fast_forward(repo, &mut r, &fetch_commit)?;
            }
            Err(_) => {
                // The branch doesn't exist so just set the reference to the
                // commit directly. Usually this is because you are pulling
                // into an empty repository.
                repo.reference(
                    &refname,
                    fetch_commit.id(),
                    true,
                    &format!("Setting {} to {}", remote_branch, fetch_commit.id()),
                )?;
                repo.set_head(&refname)?;
                repo.checkout_head(Some(
                    git2::build::CheckoutBuilder::default()
                        .allow_conflicts(true)
                        .conflict_style_merge(true)
                        .force(),
                ))?;
            }
        };
    } else if analysis.0.is_normal() {
        // do a normal merge
        let head_commit = repo.reference_to_annotated_commit(&repo.head()?)?;
        normal_merge(&repo, &head_commit, &fetch_commit)?;
    } else {
        log::debug!("Nothing to do...");
    }
    Ok(())
}

pub fn pull(repo: &Repository, remote_name: &str, remote_branch: &str) -> Result<(), git2::Error> {
    let mut remote = repo.find_remote(remote_name)?;
    let fetch_commit = do_fetch(&repo, &[remote_branch], &mut remote)?;
    do_merge(&repo, &remote_branch, fetch_commit)
}

pub fn open_or_clone<P: AsRef<Path>>(url: &str, repo_path: P) -> Result<Repository, git2::Error> {
    log::debug!(
        "open_or_clone: url: {}, repo_path: {:?}",
        url,
        repo_path.as_ref()
    );
    if let Ok(repo) = Repository::open(&repo_path) {
        log::debug!("open_or_clone: repo opened");
        Ok(repo)
    } else {
        log::debug!("open_or_clone: cloning repo");
        for _ in 0..2 {
            let r = Repository::clone(url, &repo_path);
            if r.is_ok() {
                return r;
            }
        }
        Repository::clone(url, &repo_path)
    }
}

pub static REPO_NAME: &'static str = "model-cards";

pub fn sync_model_cards_repo<P: AsRef<Path>>(app_data_dir: P) -> anyhow::Result<ModelCardManager> {
    let repo_url: &'static str =
        option_env!("MODEL_CARDS_REPO").unwrap_or("https://github.com/moxin-org/model-cards");
    let repo_dirs = app_data_dir.as_ref().join(REPO_NAME);

    let repo = open_or_clone(repo_url, &repo_dirs)?;
    let mut r = Ok(());
    for _ in 0..2 {
        r = pull(&repo, "origin", "main");
        if r.is_ok() {
            break;
        }
    }

    if let Err(e) = r {
        log::error!("Failed to pull: {:?}", e);
        log::error!("please remove the repo({:?}) and try again", &repo_dirs);
    }

    let index_url = format!("{}/releases/download/index_release/index.json", repo_url);

    let index_list = if let Ok(remote_index) =
        reqwest::blocking::get(index_url).and_then(|r| r.json::<Vec<ModelIndex>>())
    {
        remote_index
    } else {
        let index_list = std::fs::read_to_string(repo_dirs.join("index.json"))?;
        let index_list: Vec<ModelIndex> = serde_json::from_str(&index_list)?;
        index_list
    };

    let mut indexs = HashMap::with_capacity(index_list.len());
    for index in index_list {
        indexs.insert(index.id.clone(), index);
    }

    let embedding_index =
        if let Ok(embedding_index) = std::fs::read_to_string(repo_dirs.join("embedding.json")) {
            let embedding_index: EmbeddingIndex = serde_json::from_str(&embedding_index)?;
            if !embedding_index.check_file_exist(app_data_dir.as_ref()) {
                let app_data_dir_path = app_data_dir.as_ref().to_path_buf();
                let r = std::thread::spawn(move || {
                    if let Ok(_) = embedding_index.download(&app_data_dir_path) {
                        log::debug!("Downloaded embedding model ok");
                        Some(embedding_index)
                    } else {
                        log::warn!("Failed to download embedding model");
                        None
                    }
                });
                EmbeddingState::Pending(r)
            } else {
                EmbeddingState::Finish(Some(embedding_index))
            }
        } else {
            EmbeddingState::Finish(None)
        };

    Ok(ModelCardManager {
        app_data_dir: app_data_dir.as_ref().to_path_buf(),
        embedding_index,
        indexs,
        caches: HashMap::new(),
    })
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelIndex {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub architecture: String,
    #[serde(default)]
    pub model_type: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub featured: bool,
    #[serde(default)]
    pub like_count: u32,
    #[serde(default)]
    pub download_count: u32,
}

impl ModelIndex {
    pub fn load_model_card(&self, app_data_dir: &Path) -> anyhow::Result<ModelCard> {
        let (org_name, model_name) = self
            .id
            .split_once("/")
            .ok_or(anyhow::anyhow!("Invalid model id: {}", self.id))?;

        let sub_name = if !self.name.is_empty() {
            self.name.as_str()
        } else {
            model_name
        };

        let model_card_path = app_data_dir
            .join(REPO_NAME)
            .join(org_name)
            .join(format!("{}.json", sub_name));
        let model_card = std::fs::read_to_string(model_card_path)?;
        let mut model_card: ModelCard = serde_json::from_str(&model_card)?;
        model_card.like_count = self.like_count;
        model_card.download_count = self.download_count;

        Ok(model_card)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Author {
    pub name: String,
    pub url: String,
    pub description: String,
}

impl Into<moxin_protocol::data::Author> for Author {
    fn into(self) -> moxin_protocol::data::Author {
        moxin_protocol::data::Author {
            name: self.name,
            url: self.url,
            description: self.description,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelCard {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub size: String,
    #[serde(default)]
    pub requires: String,
    #[serde(default)]
    pub architecture: String,
    pub released_at: DateTime<Utc>,
    #[serde(default)]
    pub files: Vec<RemoteFile>,
    pub prompt_template: String,
    pub reverse_prompt: String,
    pub context_size: u64,
    pub author: Author,
    #[serde(default)]
    pub like_count: u32,
    #[serde(default)]
    pub download_count: u32,
    #[serde(default)]
    pub metrics: Option<HashMap<String, f32>>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RemoteFile {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub size: String,
    #[serde(default)]
    pub quantization: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub sha256: Option<String>,
    pub download: DownloadUrls,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DownloadUrls {
    #[serde(default)]
    pub default: String,
}

impl ModelCard {
    pub fn to_model(
        remote_models: &[Self],
        conn: &rusqlite::Connection,
    ) -> rusqlite::Result<Vec<moxin_protocol::data::Model>> {
        let model_ids = remote_models
            .iter()
            .map(|m| m.id.clone())
            .collect::<Vec<_>>();
        let files = super::download_files::DownloadedFile::get_by_models(conn, &model_ids)?;

        fn to_file(
            model_id: &str,
            context_size: &u64,
            remote_files: &[RemoteFile],
            save_files: &HashMap<Arc<String>, super::download_files::DownloadedFile>,
        ) -> rusqlite::Result<Vec<moxin_protocol::data::File>> {
            let mut files = vec![];
            for remote_f in remote_files {
                let file_id = format!("{}#{}", model_id, remote_f.name);
                let downloaded_path = save_files.get(&file_id).map(|file| {
                    let file_path = Path::new(&file.download_dir)
                        .join(&file.model_id)
                        .join(&file.name);
                    file_path
                        .to_str()
                        .map(|s| s.to_string())
                        .unwrap_or_default()
                });

                let file = moxin_protocol::data::File {
                    id: file_id,
                    name: remote_f.name.clone(),
                    size: remote_f.size.clone(),
                    quantization: remote_f.quantization.clone(),
                    downloaded: downloaded_path.is_some(),
                    downloaded_path,
                    tags: remote_f.tags.clone(),
                    featured: false,
                    context_size: *context_size
                };

                files.push(file);
            }

            Ok(files)
        }

        let mut models = Vec::with_capacity(remote_models.len());

        for remote_m in remote_models {
            let model = moxin_protocol::data::Model {
                id: remote_m.id.clone(),
                name: remote_m.name.clone(),
                summary: remote_m.summary.clone(),
                size: remote_m.size.clone(),
                requires: remote_m.requires.clone(),
                architecture: remote_m.architecture.clone(),
                released_at: remote_m.released_at.clone(),
                files: to_file(&remote_m.id, &remote_m.context_size, &remote_m.files, &files)?,
                author: moxin_protocol::data::Author {
                    name: remote_m.author.name.clone(),
                    url: remote_m.author.url.clone(),
                    description: remote_m.author.description.clone(),
                },
                like_count: remote_m.like_count.clone(),
                download_count: remote_m.download_count.clone(),
                metrics: remote_m.metrics.clone().unwrap_or_default(),
            };

            models.push(model);
        }

        Ok(models)
    }
}

pub struct ModelCardManager {
    app_data_dir: PathBuf,
    embedding_index: EmbeddingState,
    indexs: HashMap<String, ModelIndex>,
    caches: HashMap<String, ModelCard>,
}

pub enum EmbeddingState {
    Pending(std::thread::JoinHandle<Option<EmbeddingIndex>>),
    Finish(Option<EmbeddingIndex>),
}

impl ModelCardManager {
    pub fn empty(app_data_dir: PathBuf) -> Self {
        Self {
            app_data_dir,
            indexs: HashMap::new(),
            caches: HashMap::new(),
            embedding_index: EmbeddingState::Finish(None),
        }
    }

    pub fn load_model_card(&mut self, index: &ModelIndex) -> anyhow::Result<ModelCard> {
        let r = self
            .caches
            .entry(index.id.clone())
            .or_insert(index.load_model_card(&self.app_data_dir)?);
        Ok(r.clone())
    }

    pub fn get_index_by_id(&self, id: &str) -> Option<&ModelIndex> {
        self.indexs.get(id)
    }

    pub fn search(
        &self,
        search_text: &str,
        limit: usize,
        offset: usize,
    ) -> reqwest::Result<Vec<ModelIndex>> {
        let search_text = search_text.trim().to_ascii_lowercase();
        Ok(self
            .indexs
            .values()
            .filter(|index| {
                (index.model_type == "instruct" || index.model_type == "chat")
                    && (index.name.to_ascii_lowercase().contains(&search_text)
                        || index.architecture.to_ascii_lowercase().contains(&search_text)
                        || index.id.to_ascii_lowercase().contains(&search_text)
                        || index.summary.to_ascii_lowercase().contains(&search_text))
            })
            .map(Clone::clone)
            .skip(offset)
            .take(limit)
            .collect::<Vec<ModelIndex>>())
    }

    pub fn get_featured_model(
        &self,
        limit: usize,
        offset: usize,
    ) -> reqwest::Result<Vec<ModelIndex>> {
        Ok(self
            .indexs
            .values()
            .filter(|index| {
                (index.model_type == "instruct" || index.model_type == "chat") && index.featured
            })
            .map(Clone::clone)
            .skip(offset)
            .take(limit)
            .collect::<Vec<ModelIndex>>())
    }

    pub fn embedding_model(&mut self) -> Option<(PathBuf, u64)> {
        match &self.embedding_index {
            EmbeddingState::Pending(res) => {
                if res.is_finished() {
                    ()
                } else {
                    return None;
                }
            }
            EmbeddingState::Finish(None) => return None,
            EmbeddingState::Finish(Some(index)) => {
                return Some((index.file_path(&self.app_data_dir), index.ctx))
            }
        };

        let state = std::mem::replace(&mut self.embedding_index, EmbeddingState::Finish(None));
        let res = match state {
            EmbeddingState::Pending(res) => res.join().unwrap(),
            EmbeddingState::Finish(_) => unreachable!(),
        };

        match res {
            Some(index) => {
                let r = (index.file_path(&self.app_data_dir), index.ctx);
                self.embedding_index = EmbeddingState::Finish(Some(index));
                Some(r)
            }
            None => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmbeddingIndex {
    id: String,
    file: String,
    ctx: u64,
    download: String,
}

impl EmbeddingIndex {
    pub fn file_path(&self, app_data_dir: &Path) -> PathBuf {
        app_data_dir.join("embedding").join(&self.file)
    }

    pub fn check_file_exist(&self, app_data_dir: &Path) -> bool {
        let file_path = self.file_path(app_data_dir);
        file_path.exists()
    }

    pub fn download(&self, app_data_dir: &Path) -> anyhow::Result<()> {
        let file_path = self.file_path(app_data_dir);

        let _ = std::fs::create_dir_all(file_path.parent().unwrap());

        let client = reqwest::blocking::Client::new();
        let mut resp = client.get(&self.download).send()?;
        let mut file = std::fs::File::create(file_path)?;
        std::io::copy(&mut resp, &mut file)?;
        Ok(())
    }
}
