"""Instrument OpenEvolve's OpenAI-compatible transport without changing prompts."""

from functools import wraps

from openevolve.llm.openai import OpenAILLM

from comparison.token_usage import openai_usage, record_usage


class UsageLoggingOpenAILLM(OpenAILLM):
    def __init__(self, model_cfg):
        super().__init__(model_cfg)
        create = self.client.chat.completions.create

        @wraps(create)
        def logged_create(*args, **kwargs):
            try:
                response = create(*args, **kwargs)
            except Exception:
                record_usage(provider="openai", model=self.model, status="error")
                raise
            raw = getattr(response, "usage", None)
            raw = raw.model_dump() if raw is not None else None
            record_usage(provider="openai", model=self.model, status="response",
                         usage=openai_usage(raw), raw_usage=raw)
            return response

        self.client.chat.completions.create = logged_create
        # Let OpenEvolve handle retries so each transport attempt is visible.
        self.client.max_retries = 0


def init_usage_logging_openai_client(model_cfg):
    return UsageLoggingOpenAILLM(model_cfg)
