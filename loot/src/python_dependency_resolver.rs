use pyo3::prelude::*;
use std::collections::HashMap;

pub fn resolve_dependencies(
    dependencies: &HashMap<String, String>,
) -> PyResult<Vec<(String, String)>> {
    Python::with_gil(|py| {
        let dependencies_to_add = dependencies
            .iter()
            .map(|(name, version)| {
                format!(
                    "packaging.requirements.Requirement('{}=={}')",
                    name, version
                )
            })
            .collect::<Vec<String>>()
            .join(",");

        let code = include_str!("./resolve_dependencies.py");
        let code = code.replace("# START", &dependencies_to_add);

        py.run_bound(&code, None, None)?;

        let result: Vec<(String, String)> = py.eval_bound("result", None, None)?.extract()?;

        Ok(result)
    })
}
