#!/usr/bin/envs python3
# -*- coding: utf-8 -*-
import json
from dora import DoraStatus
import pyarrow as pa

from mae.kernel.utils.log import write_agent_log
from mae.kernel.utils.util import load_agent_config
from mae.run.run import run_dspy_agent, run_crewai_agent



class Operator:
    def on_event(
            self,
            dora_event,
            send_output,
    ) -> DoraStatus:
        if dora_event["type"] == "INPUT":
            if dora_event['id'] == 'feedback_result':
                inputs = load_agent_config('use_case/refinement_agent.yml')
                result = """
                """
                dora_result = json.loads(dora_event["value"][0].as_py())
                inputs['context'] = dora_result.get('context')
                inputs['input_fields'] = {'suggestion': dora_result.get('suggestion'),
                                          'rag_data':json.dumps(dora_result.get('rag_data'))}
                print(inputs)
                if 'agents' not in inputs.keys():
                    inputs['task'] = dora_result['task']
                    result = run_dspy_agent(inputs=inputs)
                else:
                    result = run_crewai_agent(crewai_config=inputs)
                if inputs.get('max_iterations',None) is not None:

                    result = {'task':dora_result.get('task'),'suggestion':result,'context':result,'local_iterations':dora_result.get('local_iterations', None),'rag_data':dora_result.get('rag_data')}
                else:
                    result = {'task':dora_result.get('task'),'suggestion':result,'context':result,'rag_data':dora_result.get('rag_data')}
                print(result)
                log_result = {"6, " + inputs.get('log_step_name', "Step_one"): result['context']}
                write_agent_log(log_type=inputs.get('log_type', None), log_file_path=inputs.get('log_path', None),
                                data=log_result)
                send_output("refinement_result", pa.array([json.dumps(result)]),dora_event['metadata'])  # add this line

        return DoraStatus.CONTINUE



