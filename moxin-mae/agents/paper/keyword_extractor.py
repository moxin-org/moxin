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
            if dora_event["id"] == "task":
                task = dora_event["value"][0].as_py()
                config_values = json.loads(dora_event["value"][1].as_py())
                inputs = load_agent_config('use_case/keyword_extractor.yml')

                # Use provided API key that comes from Moxin
                inputs['model_api_key'] = config_values["model_api_key"]

                result = """
                                """
                if 'agents' not in inputs.keys():
                    inputs['task'] = task
                    result = run_dspy_agent(inputs=inputs)
                else:
                    result = run_crewai_agent(crewai_config=inputs)
                log_result = {"1, "+ inputs.get('log_step_name',"Step_one"):{task:result}}
                write_agent_log(log_type=inputs.get('log_type',None),log_file_path=inputs.get('log_path',None),data=log_result)
                result_dict = {'task':task,'keywords':result}
                print(result_dict)

                # Carry on Moxin config values from previous step
                send_output("keywords", pa.array([json.dumps(result_dict), json.dumps(config_values)]),dora_event['metadata'])

        return DoraStatus.CONTINUE