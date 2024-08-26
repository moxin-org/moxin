import json

from dora import Node, DoraStatus
import pyarrow as pa
from pathlib import Path

from mae.kernel.utils.log import write_agent_log
from mae.kernel.utils.util import load_agent_config
from mae.run.run import run_dspy_agent, run_crewai_agent
from mae.utils.files.read import read_yaml
from mae.utils.files.util import get_all_files


class Operator:
    def on_event(
        self,
        dora_event,
        send_output,
    ) -> DoraStatus:
        if dora_event["type"] == "INPUT":
            if dora_event['id'] == 'papers_info':
                config = {}
                inputs = load_agent_config('use_case/paper_analyze_agent.yml')
                paper_result = json.loads(dora_event["value"][0].as_py())

                result = """ """
                all_result = []
                for file_path in  inputs.get('files_path'):

                    if Path(file_path).is_dir():
                        files_path = list(get_all_files(file_path))
                        for local_file_path in files_path:
                            try:
                                inputs['files_path'] = [local_file_path]
                                inputs['collection_name'] = Path(local_file_path).name
                                if 'agents' not in inputs.keys():
                                    result = run_dspy_agent(inputs=inputs)
                                else:
                                    result = run_crewai_agent(crewai_config=inputs)
                                print(inputs)
                                all_result.append({local_file_path:result})
                                print('local_file_rag_summary    : ' , {local_file_path:result,'inputs':inputs})
                            except Exception as e :
                                print('------- inputs: ',inputs)
                                print('pdf analysis appearance problem  :',e)
                                continue

                    else:
                        if 'agents' not in inputs.keys():
                            result = run_dspy_agent(inputs=inputs)
                        else:
                            result = run_crewai_agent(crewai_config=inputs)
                        all_result.append(result)

                log_result = {"3, " + inputs.get('log_step_name', "Step_one"): {k.split('/')[-1]: v for d in all_result for k, v in d.items()}}
                write_agent_log(log_type=inputs.get('log_type', None), log_file_path=inputs.get('log_path', None),
                                data=log_result)
                result_dict = {'task':paper_result.get('task'),'context':all_result}
                print(result_dict)
                send_output("paper_analyze_result", pa.array([json.dumps(result_dict)]),dora_event['metadata'])
                return DoraStatus.STOP
        return DoraStatus.CONTINUE



