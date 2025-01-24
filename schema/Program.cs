using NJsonSchema;
using NJsonSchema.CodeGeneration;
using NJsonSchema.CodeGeneration.CSharp;
using System;
using System.IO;

if (args.Length != 2)
{
    Console.WriteLine("Usage: program <input-schema.json> <output-file.cs>");
    return 1;
}

try
{
    // Read and parse the JSON schema
    var schemaJson = File.ReadAllText(args[0]);
    var schema = JsonSchema.FromJsonAsync(schemaJson).GetAwaiter().GetResult();

    // Generate C# code with custom settings
    var settings = new CSharpGeneratorSettings
    {
        GenerateOptionalPropertiesAsNullable = true,
        GenerateNullableReferenceTypes = true
    };
    var generator = new CSharpGenerator(schema, settings);
    var fileContent = generator.GenerateFile();

    // Write the generated code to the output file
    File.WriteAllText(args[1], fileContent);

    Console.WriteLine($"Successfully generated C# code to {args[1]}");
    return 0;
}
catch (Exception ex)
{
    Console.WriteLine($"Error: {ex.Message}");
    return 1;
}
