import json
import os
from dora import Node, DoraStatus
import pyarrow as pa
from mae.kernel.utils.log import write_agent_log
from mae.kernel.utils.util import load_agent_config
from mae.run.run import run_dspy_agent, run_crewai_agent
from mae.utils.files.dir import get_relative_path
from mae.utils.files.read import read_yaml



class Operator:
    def on_event(
        self,
        dora_event,
        send_output,
    ) -> DoraStatus:
        if dora_event["type"] == "INPUT":
            agent_inputs = ['reasoner_task','task_input']
            if dora_event["id"] in agent_inputs:
                print(f'config:   {dora_event}')
                # dora_result = json.loads(dora_event["value"][0].as_py())
                # task_inputs = json.loads(dora_event["value"][0].as_py())

                task_inputs = dora_event["value"][0].as_py()
                print(f'config:   {task_inputs}')
                if isinstance(task_inputs, dict):
                    task = task_inputs.get('task', None)
                else: task = task_inputs
                yaml_file_path = get_relative_path(current_file=__file__, sibling_directory_name='configs', target_file_name='reasoner_agent.yml')
                inputs = load_agent_config(yaml_file_path)
                if inputs.get('check_log_prompt', None) is True:
                    log_config = {}
                    agent_config =  read_yaml(yaml_file_path).get('AGENT', '')
                    agent_config['task'] = task
                    log_config[' Agent Prompt'] = agent_config
                    write_agent_log(log_type=inputs.get('log_type', None), log_file_path=inputs.get('log_path', None),
                                    data=log_config)
                result = """
                                """
                if 'agents' not in inputs.keys():
                    inputs['task'] = task
                    result = run_dspy_agent(inputs=inputs)
                else:
                    result = run_crewai_agent(crewai_config=inputs)
                print(f'config:   {inputs}')
                log_result = {inputs.get('log_step_name', "Step_one") :result}
                results = {}
                write_agent_log(log_type=inputs.get('log_type',None),log_file_path=inputs.get('log_path',None),data=log_result)
                results['task'] = task
                results['result'] = result
                print('agent_output:',results)
                send_output("reasoner_result", pa.array([json.dumps(results)]),dora_event['metadata'])
                return DoraStatus.CONTINUE

        return DoraStatus.CONTINUE