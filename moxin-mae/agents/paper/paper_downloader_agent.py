import json

from dora import Node, DoraStatus
import pyarrow as pa

from mae.kernel.utils.log import write_agent_log
from mae.kernel.utils.util import load_agent_config
from mae.run.run import run_dspy_agent, run_crewai_agent
from mae.utils.files.read import read_yaml



class Operator:
    def on_event(
        self,
        dora_event,
        send_output,
    ) -> DoraStatus:
        if dora_event["type"] == "INPUT":
            if dora_event['id'] == 'keywords':
                inputs = load_agent_config('use_case/paper_downloader_agent.yml')
                keyword_result = json.loads(dora_event["value"][0].as_py())
                config_values = json.loads(dora_event["value"][1].as_py())
                print('inputs   : ',inputs)
                result = """
"""
                inputs.get('tasks')[0]['description'] = f"keywords: {keyword_result.get('keywords')}"

                # Use provided API key that comes from Moxin
                inputs.get('model')['model_api_key'] = config_values["model_api_key"]
                inputs.get('env')['SERPER_API_KEY'] = config_values["serper_api_key"]
                inputs.get('env')['AGENTOPS_API_KEY'] = config_values["agentops_api_key"]

                print('inputs with keys  : ',inputs)

                if 'agents' not in inputs.keys():
                    result = run_dspy_agent(inputs=inputs)
                else:
                    result = run_crewai_agent(crewai_config=inputs)

                print('result  : ',result)
                log_config = inputs.get('log')
                log_result =  {"2, "+log_config.get('log_step_name',"Step_one"):result}
                write_agent_log(log_type=log_config.get('log_type',None),log_file_path=log_config.get('log_path',None),data=log_result)

                result_dict = {'task':keyword_result.get('task')}
                # Carry on Moxin config values from previous step
                send_output("papers_info", pa.array([json.dumps(result_dict), json.dumps(config_values)]),dora_event['metadata'])
        return DoraStatus.CONTINUE