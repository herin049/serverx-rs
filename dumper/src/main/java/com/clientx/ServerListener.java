package com.clientx;

import com.clientx.dumpers.BiomeDumper;
import com.clientx.dumpers.BlockDumper;
import com.clientx.dumpers.Dumper;
import com.clientx.dumpers.RegistryDumper;
import com.google.gson.GsonBuilder;
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerTickEvents;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.world.ServerWorld;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import static com.clientx.DataDumper.LOGGER;

public class ServerListener implements ServerTickEvents.EndTick {

    @Override
    public void onEndTick(MinecraftServer server) {
        var dumpers = new Dumper[] {
                new BlockDumper("blocks.json"),
                new BiomeDumper("biomes.json", server),
                new RegistryDumper("registries.nbt", server)
        };
        Path outputDirectory;
        try {
            outputDirectory = Files.createDirectories(Paths.get("dumper_out"));
        } catch (IOException e) {
            LOGGER.error("Failed to create output directory.", e);
            return;
        }

        var gson = new GsonBuilder().setPrettyPrinting().disableHtmlEscaping().serializeNulls().create();

        for (var dumper : dumpers) {
            dumper.dump(gson, outputDirectory);
        }
        server.shutdown();
    }
}
