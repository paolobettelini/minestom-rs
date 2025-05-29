package net.thecrown;

import net.worldseed.resourcepack.PackBuilder;
import org.apache.commons.io.FileUtils;

import java.io.IOException;
import java.nio.charset.Charset;
import java.nio.file.Path;
import java.nio.file.Paths;

public class PackHelper {

    /** 
     * Called by your build.rs via Gradle.
     * @param bbmodelPath        path to the bbmodel directory/file
     * @param resourcepackPath   path to the generated resourcepack dir
     * @param modelsDirPath      path to the models output dir
     */
    public static void generate(String bbmodelPath,
                                String resourcepackPath,
                                String modelsDirPath,
                                String mappingsPath) {
        Path bbmodelDir = Paths.get(bbmodelPath);
        Path resourcepackDir = Paths.get(resourcepackPath);
        Path modelsDir = Paths.get(modelsDirPath);
        Path mappings = Paths.get(mappingsPath);

        System.out.println("Generating pack:");
        System.out.println("  bbmodel = " + bbmodelPath);
        System.out.println("  resourcepack = " + resourcepackPath);
        System.out.println("  modelsDir = " + modelsDir);
        System.out.println("  mappings = " + mappings);
    
        try {
            var config = PackBuilder.Generate(bbmodelDir, resourcepackDir, modelsDir);
            FileUtils.writeStringToFile(
                mappings.toFile(),
                config.modelMappings(),
                Charset.defaultCharset()
            );
        } catch (Exception e) {
            System.err.println("Error generating pack: " + e.getMessage());
            e.printStackTrace();
            System.exit(1);
            return;
        }

    }

    public static void main(String[] args) {
        if (args.length != 4) {
            System.err.println("Usage: PackBuilder <bbmodel> <respackDir> <modelsDir> <mappingsPath>");
            System.exit(1);
        }
        generate(args[0], args[1], args[2], args[3]);
    }
    
}
