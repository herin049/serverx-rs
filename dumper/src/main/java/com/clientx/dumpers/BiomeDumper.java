package com.clientx.dumpers;

import com.google.gson.Gson;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import net.minecraft.registry.RegistryKey;
import net.minecraft.registry.RegistryKeys;
import net.minecraft.server.MinecraftServer;
import net.minecraft.world.biome.Biome;
import net.minecraft.world.biome.BiomeEffects;

import java.io.FileWriter;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;

import static com.clientx.DataDumper.LOGGER;

public class BiomeDumper implements Dumper {
    private String fileName;
    private MinecraftServer server;
    public BiomeDumper(String fileName, MinecraftServer server) {
        this.fileName = fileName;
        this.server = server;
    }
    @Override
    public void dump(Gson gson, Path path) {
        var biomeRegistry = server.getRegistryManager().get(RegistryKeys.BIOME);
        var resultJson = new JsonObject();
        var biomesJson = new JsonArray();
        for (var biome: biomeRegistry) {
           var biomeJson = new JsonObject();
           biomeJson.addProperty("id", biomeRegistry.getRawId(biome));
           biomeJson.addProperty("name", biomeRegistry.getId(biome).getPath());
           biomeJson.addProperty("has_precipitation", biome.hasPrecipitation());
           biomeJson.addProperty("temperature", biome.getTemperature());

           var biomeEffects = biome.getEffects();
           var biomeEffectsJson = new JsonObject();
           biomeEffectsJson.addProperty("fog_color", biomeEffects.getFogColor());
           biomeEffectsJson.addProperty("water_color", biomeEffects.getWaterColor());
           biomeEffectsJson.addProperty("water_fog_color", biomeEffects.getWaterFogColor());
           biomeEffectsJson.addProperty("sky_color", biomeEffects.getSkyColor());
           biomeEffects.getFoliageColor().ifPresent(c -> biomeEffectsJson.addProperty("foliage_color", c));
           biomeEffects.getGrassColor().ifPresent(c -> biomeEffectsJson.addProperty("grass_color", c));
           biomeEffectsJson.addProperty("grass_color_modifier", biomeEffects.getGrassColorModifier().asString());
           biomeJson.add("effects", biomeEffectsJson);

           biomesJson.add(biomeJson);
        }
        resultJson.add("biomes", biomesJson);

        var grassColorModifiersJson = new JsonArray();
        for (var color: BiomeEffects.GrassColorModifier.values()) {
            grassColorModifiersJson.add(color.asString());
        }

        resultJson.add("grass_color_modifiers", grassColorModifiersJson);

        try {
            var fileWriter = new FileWriter(path.resolve(fileName).toFile(), StandardCharsets.UTF_8);
            gson.toJson(resultJson, fileWriter);
            fileWriter.close();
        } catch (Exception e) {
            LOGGER.error("Failed to save blocks output", e);
        }
    }
}
