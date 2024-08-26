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
            if dora_event['id'] == 'paper_analyze_result':
                inputs = load_agent_config('use_case/report_writer_agent.yml')
                paper_analyze = json.loads(dora_event["value"][0].as_py())
                inputs['context'] = paper_analyze.get('context')
                inputs['task'] = paper_analyze.get('task')
                # result = """ "Improve Pre-training Methods:\n\n1. **Knowledge Integration**: During the pre-training process, integrate entity embeddings from knowledge bases, and combine entity linking loss with masked language modeling (MLM) loss to enhance the model's knowledge recall ability [2304.01597v1].\n\n2. **Self-supervised Learning**: Utilize a larger scale of unlabeled data, and improve the model's language understanding and generation capabilities through self-supervised learning [2304.01597v1].\n\nEnhance Fine-tuning Techniques:\n\n1. **Task-specific Adaptation**: Implement task-specific fine-tuning strategies that adapt the model to particular domains or tasks, improving its performance in specialized areas [2305.01234v2].\n\n2. **Multi-task Learning**: Employ multi-task learning approaches where the model is fine-tuned on multiple related tasks simultaneously, leading to better generalization and transfer learning capabilities [2305.01234v2].\n\nIncorporate Advanced Architectures:\n\n1. **Hybrid Models**: Develop hybrid models that combine the strengths of different architectures, such as transformers and recurrent neural networks (RNNs), to leverage their complementary advantages [2306.04567v1].\n\n2. **Sparse Attention Mechanisms**: Introduce sparse attention mechanisms to reduce computational complexity and enhance the scalability of large language models [2306.04567v1].\n\nLeverage External Knowledge Sources:\n\n1. **Knowledge Distillation**: Use knowledge distillation techniques to transfer knowledge from larger, more complex models to smaller, more efficient ones without significant loss in performance [2307.02345v1].\n\n2. **External Memory Integration**: Integrate external memory modules that allow the model to access and retrieve information from large-scale knowledge bases dynamically [2307.02345v1].\n\nMonitor and Adapt to Emerging Trends:\n\n1. **Continuous Learning**: Implement continuous learning frameworks that enable the model to adapt to new data and evolving language patterns over time [2308.03456v1].\n\n2. **Ethical and Fairness Considerations**: Address ethical and fairness issues by incorporating bias detection and mitigation strategies during both pre-training and fine-tuning phases [2308.03456v1].\n\nBy synthesizing these diverse approaches, researchers can enhance the capabilities of mixed large language models, ensuring they remain relevant, efficient, and effective in a rapidly evolving field." """
                print(inputs)
                if 'agents' not in inputs.keys():

                    result = run_dspy_agent(inputs=inputs)
                else:
                    result = run_crewai_agent(crewai_config=inputs)
                if inputs.get('max_iterations',None) is not None:
                    max_iterations  = inputs.get('max_iterations',None)
                    result = {'task':paper_analyze.get('task'),'max_iterations': max_iterations,'context':result,'local_iterations':1,'rag_data':paper_analyze.get('context')}
                else:
                    result = { 'task':paper_analyze.get('task'),'context': result,'local_iterations':1,'rag_data':paper_analyze.get('context')}
                print(result)
                log_result = {"4, " +  inputs.get('log_step_name', "Step_one"): result['context']}
                write_agent_log(log_type=inputs.get('log_type', None), log_file_path=inputs.get('log_path', None),
                                data=log_result)
                send_output("writer_report", pa.array([json.dumps(result)]),dora_event['metadata'])  # add this line
                return DoraStatus.STOP
        return DoraStatus.CONTINUE



