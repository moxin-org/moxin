import json

from dora import Node
import pyarrow as pa

node = Node("paper_task_input")

event = node.next()
task_data = input('Please enter your task:  ',)
node.send_output('task',pa.array([json.dumps(task_data)]),event["metadata"])