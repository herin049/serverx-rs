from jinja2 import Environment, Template, FileSystemLoader
from collections import defaultdict
import json
from caseconverter import pascalcase


def render(env: Environment) -> str:
    template: Template = env.get_template("blocks/blocks.rs.j2")
    with open("data/blocks.json") as file:
        blocks = json.loads(file.read())
    for b in blocks["blocks"]:
        for p in b.get("properties", []):
            p["enum_name"] = pascalcase(p["name"]) + "Property"
    return template.render(blocks=blocks["blocks"], pascalcase=pascalcase)


if __name__ == "__main__":
    env = Environment(
        loader=FileSystemLoader("templates"), trim_blocks=True, lstrip_blocks=True
    )
    result = render(env)
    with open("output/blocks/blocks.rs", "w") as file:
        file.write(result)
