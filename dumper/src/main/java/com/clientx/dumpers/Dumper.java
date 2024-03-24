package com.clientx.dumpers;

import com.google.gson.Gson;
import com.google.gson.JsonObject;

import java.nio.file.Path;

public interface Dumper {
    public void dump(Gson gson, Path path);
}
