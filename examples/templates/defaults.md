> **🤖 `{{ runtime.agent.name }}`   📦 `{{ runtime.project.name }}`   💬 `{{ runtime.session.name }}`**
> **🏠 `{{ runtime.user }}@{{ runtime.hostname }}`   📅 `{{ runtime.timestamp.local }}`{% if runtime.session.id %}   🧵 `{{ runtime.session.id }}`{% endif %}**
{{ message }}
