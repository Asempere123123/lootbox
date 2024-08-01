import resolvelib
import resolver
import packaging.requirements

resolver_instance = resolvelib.Resolver(
    resolver.Provider(),
    resolvelib.BaseReporter(),
)

resolved_deps = resolver_instance.resolve([
# START
]).mapping.items()

result = []
for (_, value) in resolved_deps:
    result.append((value.name, str(value.version)))
