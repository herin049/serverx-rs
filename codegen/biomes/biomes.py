from jinja2 import Environment, Template, FileSystemLoader
from collections import defaultdict
import json
from caseconverter import pascalcase


def render(env: Environment) -> str:
    template: Template = env.get_template("biomes/biomes.rs.j2")
    with open("data/biomes.json") as file:
        biomes = json.loads(file.read())
    return template.render(
        biomes=biomes["biomes"],
        biome_count=len(biomes["biomes"]),
        grass_color_modifiers=biomes["grass_color_modifiers"],
        pascalcase=pascalcase,
    )


if __name__ == "__main__":
    env = Environment(
        loader=FileSystemLoader("templates"), trim_blocks=True, lstrip_blocks=True
    )
    result = render(env)
    with open("output/biomes/biomes.rs", "w") as file:
        file.write(result)
