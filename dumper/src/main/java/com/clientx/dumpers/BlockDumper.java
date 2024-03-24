package com.clientx.dumpers;

import com.google.gson.Gson;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import net.minecraft.registry.Registries;
import net.minecraft.state.property.*;
import net.minecraft.util.math.BlockPos;
import net.minecraft.world.EmptyBlockView;

import java.io.BufferedWriter;
import java.io.FileWriter;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Collection;
import java.util.LinkedHashMap;
import java.util.Locale;
import java.util.Objects;

import static com.clientx.DataDumper.LOGGER;

public class BlockDumper implements Dumper {

    private String fileName;

    public BlockDumper(String fileName) {
        this.fileName = fileName;
    }

    private String getPropertyName(Property property) {
        if(property.getName().equals("type")) {
            return "kind";
        } else {
            return property.getName();
        }
    }

    @Override
    public void dump(Gson gson, Path path) {
        var blockStateId = 0;
        var resultJson = new JsonObject();
        var blocksJson = new JsonArray();
        var shapes = new LinkedHashMap<Shape, Integer>();

        for (var block: Registries.BLOCK) {
            var blockJson = new JsonObject();
            blockJson.addProperty("id", Registries.BLOCK.getRawId(block));
            blockJson.addProperty("name", Registries.BLOCK.getId(block).getPath());
            blockJson.addProperty("translation_key", block.getTranslationKey());
            blockJson.addProperty("item_id", Registries.ITEM.getRawId(block.asItem()));

            var propertiesJson = new JsonArray();
            for (var prop : block.getStateManager().getProperties()) {
                var propJson = new JsonObject();
                var propName = getPropertyName(prop);
                propJson.addProperty("name", propName);
                if (prop instanceof BooleanProperty) {
                    propJson.addProperty("type", "boolean");
                } else if (prop instanceof IntProperty) {
                    propJson.addProperty("type", "integer");
                } else {
                    propJson.addProperty("type", "enum");
                }

                var valuesJson = new JsonArray();
                for (var value : prop.getValues()) {
                    if(prop instanceof BooleanProperty) {
                        valuesJson.add((Boolean) value);
                    } else if (prop instanceof DirectionProperty) {
                        valuesJson.add(value.toString().toLowerCase(Locale.ROOT));
                    } else if (prop instanceof EnumProperty<?>) {
                        valuesJson.add(value.toString().toLowerCase(Locale.ROOT));
                    } else if (prop instanceof IntProperty) {
                        valuesJson.add((Integer) value);
                    }
                }
                propJson.add("values", valuesJson);
                propertiesJson.add(propJson);
            }
            blockJson.add("properties", propertiesJson);
            var statesJson = new JsonArray();
            for (var state : block.getStateManager().getStates()) {
                var stateJson = new JsonObject();
                var id = blockStateId++;
                stateJson.addProperty("id", id);
                stateJson.addProperty("luminance", state.getLuminance());
                stateJson.addProperty("opaque", state.isOpaque());
                stateJson.addProperty("replaceable", state.isReplaceable());
                // TODO `blocksMovement` seems to be deprecated. How else can one get this property?
                stateJson.addProperty("blocks_motion", state.blocksMovement());
                stateJson.addProperty("is_air", state.isAir());

                if (block.getDefaultState().equals(state)) {
                    blockJson.addProperty("default_state_id", id);
                }

                var statePropertiesJson = new JsonObject();
                for (var e : state.getEntries().entrySet()) {
                    var propName = getPropertyName(e.getKey());
                    if (e.getValue() instanceof Boolean b) {
                        statePropertiesJson.addProperty(propName, b);
                    } else if (e.getValue() instanceof Integer i) {
                        statePropertiesJson.addProperty(propName, i);
                    } else {
                        statePropertiesJson.addProperty(propName, e.getValue().toString().toLowerCase(Locale.ROOT));
                    }
                }
                stateJson.add("properties", statePropertiesJson);

                var collisionShapeIdxsJson = new JsonArray();
                for (var box : state.getCollisionShape(EmptyBlockView.INSTANCE, BlockPos.ORIGIN).getBoundingBoxes()) {
                    var collisionShape = new Shape(box.minX, box.minY, box.minZ, box.maxX, box.maxY, box.maxZ);

                    var idx = shapes.putIfAbsent(collisionShape, shapes.size());
                    collisionShapeIdxsJson.add(Objects.requireNonNullElseGet(idx, () -> shapes.size() - 1));
                }

                stateJson.add("collision_shapes", collisionShapeIdxsJson);

                for (var blockEntity : Registries.BLOCK_ENTITY_TYPE) {
                    if (blockEntity.supports(state)) {
                        stateJson.addProperty("block_entity_type", Registries.BLOCK_ENTITY_TYPE.getRawId(blockEntity));
                    }
                }

                statesJson.add(stateJson);
            }

            blockJson.add("states", statesJson);
            blocksJson.add(blockJson);
        }

        var blockEntitiesJson = new JsonArray();
        for (var blockEntity : Registries.BLOCK_ENTITY_TYPE) {
            var blockEntityJson = new JsonObject();
            blockEntityJson.addProperty("id", Registries.BLOCK_ENTITY_TYPE.getRawId(blockEntity));
            blockEntityJson.addProperty("ident", Registries.BLOCK_ENTITY_TYPE.getId(blockEntity).toString());
            blockEntityJson.addProperty("name", Registries.BLOCK_ENTITY_TYPE.getId(blockEntity).getPath());

            blockEntitiesJson.add(blockEntityJson);
        }

        var shapesJson = new JsonArray();
        for (var shape : shapes.keySet()) {
            var shapeJson = new JsonObject();
            shapeJson.addProperty("min_x", shape.minX);
            shapeJson.addProperty("min_y", shape.minY);
            shapeJson.addProperty("min_z", shape.minZ);
            shapeJson.addProperty("max_x", shape.maxX);
            shapeJson.addProperty("max_y", shape.maxY);
            shapeJson.addProperty("max_z", shape.maxZ);
            shapesJson.add(shapeJson);
        }

        resultJson.add("block_entity_types", blockEntitiesJson);
        resultJson.add("shapes", shapesJson);
        resultJson.add("blocks", blocksJson);

        try {
            var fileWriter = new FileWriter(path.resolve(fileName).toFile(), StandardCharsets.UTF_8);
            gson.toJson(resultJson, fileWriter);
            fileWriter.close();
        } catch (Exception e) {
            LOGGER.error("Failed to save blocks output", e);
        }
    }


    private record Shape(double minX, double minY, double minZ, double maxX, double maxY, double maxZ) {
    }
}
