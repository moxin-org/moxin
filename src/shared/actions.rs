use makepad_widgets::DefaultNone;
use moxin_mae::MaeAgent;
use moxin_protocol::data::FileID;

#[derive(Clone, Debug)]
pub enum ChatHandler {
    Model(FileID),
    Agent(MaeAgent),
}

#[derive(Clone, DefaultNone, Debug)]
pub enum ChatAction {
    Start(ChatHandler),
    None,
}

#[derive(Clone, DefaultNone, Debug)]
pub enum DownloadAction {
    Play(FileID),
    Pause(FileID),
    Cancel(FileID),
    None,
}
