from jinja2 import Environment, Template, FileSystemLoader
from collections import defaultdict
import json
from caseconverter import pascalcase


def render(env: Environment) -> str:
    template: Template = env.get_template("blocks/states.rs.j2")
    with open("data/blocks.json") as file:
        blocks = json.loads(file.read())
    for b in blocks["blocks"]:
        for p in b.get("properties", []):
            pname = p["name"]
        for s in b["states"]:
            s["block"] = dict(
                filter(lambda e: e[0] in ("id", "name", "default_state_id"), b.items())
            )
            props = s.get("properties", {})
            for k, v in props.items():
                if type(v) is bool:
                    v = "true" if v else "false"
                elif type(v) is str:
                    v = pascalcase(k) + "Property::" + pascalcase(v)
                props[k] = v
    states = []
    for b in blocks["blocks"]:
        for s in b["states"]:
            states.append(s)
    return template.render(
        states=states,
        blocks=blocks["blocks"],
        state_count=len(states),
        pascalcase=pascalcase,
    )


if __name__ == "__main__":
    env = Environment(
        loader=FileSystemLoader("templates"), trim_blocks=True, lstrip_blocks=True
    )
    result = render(env)
    with open("output/blocks/states.rs", "w") as file:
        file.write(result)
