package com.clientx;

import com.clientx.dumpers.BlockDumper;
import com.clientx.dumpers.Dumper;
import com.google.gson.GsonBuilder;
import net.fabricmc.api.ModInitializer;

import net.fabricmc.fabric.api.event.lifecycle.v1.ServerTickEvents;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.FileWriter;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;

public class DataDumper implements ModInitializer {
    public static final Logger LOGGER = LoggerFactory.getLogger("datadumper");

	@Override
	public void onInitialize() {
		ServerTickEvents.END_SERVER_TICK.register(new ServerListener());
	}
}