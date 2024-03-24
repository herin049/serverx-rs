package com.clientx.dumpers;

import com.google.gson.Gson;
import io.netty.handler.codec.EncoderException;
import net.minecraft.nbt.NbtElement;
import net.minecraft.nbt.NbtIo;
import net.minecraft.nbt.NbtOps;
import net.minecraft.registry.*;
import net.minecraft.server.MinecraftServer;
import net.minecraft.util.Util;

import java.io.DataOutputStream;
import java.io.FileOutputStream;
import java.io.IOException;
import java.nio.file.Path;

public class RegistryDumper implements Dumper {
    private String fileName;
    private MinecraftServer server;
    public RegistryDumper(String fileName, MinecraftServer server) {
        this.fileName = fileName;
        this.server = server;
    }
    @Override
    public void dump(Gson gson, Path path) {
        CombinedDynamicRegistries<ServerDynamicRegistryType> combinedDynamicRegistries = server.getCombinedDynamicRegistries();
        DynamicRegistryManager.Immutable registryManager = new DynamicRegistryManager.ImmutableImpl(SerializableRegistries.streamDynamicEntries(combinedDynamicRegistries)).toImmutable();
        RegistryOps<NbtElement> vanillaRegistryOps = RegistryOps.of(NbtOps.INSTANCE, DynamicRegistryManager.of(Registries.REGISTRIES));
        NbtElement nbtElement = Util.getResult(SerializableRegistries.CODEC.encodeStart(vanillaRegistryOps, registryManager), error -> new EncoderException("Failed to encode: " + error + " " + registryManager));
        try {
            DataOutputStream dataOutputStream =
                    new DataOutputStream(
                            new FileOutputStream(path.resolve(fileName).toString()));
            NbtIo.write(nbtElement, dataOutputStream);
            dataOutputStream.flush();
            dataOutputStream.close();
        } catch (IOException e) {
            e.printStackTrace();
        }
    }
}
