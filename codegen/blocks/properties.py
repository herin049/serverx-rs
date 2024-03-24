from jinja2 import Environment, Template, FileSystemLoader
from collections import defaultdict
import json
from caseconverter import pascalcase


def render(env: Environment) -> str:
    template: Template = env.get_template("blocks/properties.rs.j2")
    with open("data/blocks.json") as file:
        blocks = json.loads(file.read())
    properties_dict = defaultdict(lambda: set())
    for block in blocks["blocks"]:
        for prop in block.get("properties", []):
            if prop["type"] != "enum":
                continue
            for value in prop["values"]:
                properties_dict[pascalcase(prop["name"]) + "Property"].add(
                    pascalcase(value)
                )
    for k, v in properties_dict.items():
        properties_dict[k] = sorted(list(v))
    return template.render(properties=properties_dict)


if __name__ == "__main__":
    env = Environment(
        loader=FileSystemLoader("templates"), trim_blocks=True, lstrip_blocks=True
    )
    result = render(env)
    with open("output/blocks/properties.rs", "w") as file:
        file.write(result)
