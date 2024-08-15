import json
import pyarrow as pa
from dora import DoraStatus


class Operator:
    def on_event(
        self,
        dora_event,
        send_output,
    ) -> DoraStatus:
        if dora_event["type"] == "INPUT":
            agent_inputs = ['reasoner_result']
            if dora_event['id'] in agent_inputs:
                input = dora_event["value"][0].as_py()
                print(f'agent_output   : {json.loads(input)}')
                send_output("reasoner_output", pa.array([json.dumps(input)]), dora_event['metadata'])
            # return DoraStatus.CONTINUE
        return DoraStatus.CONTINUE