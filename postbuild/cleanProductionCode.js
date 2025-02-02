import prettier from "prettier";
import { glob } from "glob";
import fs from "fs/promises";
import path from "path";

// Core cleaning function remains the same but with better error handling
function cleanProductionCode(sourceCode) {
  try {
    // Split the code into lines for processing
    let lines = sourceCode.split("\n");
    let cleanedLines = [];

    for (let i = 0; i < lines.length; i++) {
      const currentLine = lines[i];

      // Case 3: Handle "Add in production" comments
      if (currentLine.includes("// CRITICAL: Add in production!")) {
        const contentToAdd = currentLine.split("!")[1];
        if (contentToAdd) {
          cleanedLines.push(contentToAdd.trim());
        }
        continue;
      }

      // Case 1: Skip lines with "Remove in production" comment
      if (currentLine.includes("// CRITICAL: Remove in production")) {
        continue;
      }

      // Case 2: Check for multi-line blocks ending with "Remove in production"
      if (currentLine.includes("// CRITICAL: Remove in production!")) {
        // Remove the previous lines that were part of this block
        let j = cleanedLines.length - 1;
        let openBraces = 0;

        // Count closing braces/parentheses in current line
        const closingCount = (currentLine.match(/[})\]]/g) || []).length;

        // Remove lines until we find the matching opening structure
        while (j >= 0) {
          const line = cleanedLines[j];
          const openCount = (line.match(/[{([]/g) || []).length;
          const closeCount = (line.match(/[})\]]/g) || []).length;

          openBraces += openCount - closeCount;

          if (openBraces >= closingCount) {
            cleanedLines = cleanedLines.slice(0, j);
            break;
          }
          j--;
        }
        continue;
      }

      // Add the line if it wasn't skipped
      cleanedLines.push(currentLine);
    }

    // Join the lines back together
    let cleanedCode = cleanedLines.join("\n");

    // Format the code using prettier
    try {
      cleanedCode = prettier.format(cleanedCode, {
        parser: "typescript",
        semi: true,
        singleQuote: true,
        tabWidth: 4,
      });
    } catch (error) {
      console.warn(`Warning: Prettier formatting failed: ${error.message}`);
      // Return the unformatted but cleaned code
      return cleanedCode;
    }

    return cleanedCode;
  } catch (error) {
    throw new Error(`Error cleaning code: ${error.message}`);
  }
}

// New function to process multiple files based on patterns
async function processFiles(patterns) {
  try {
    // Convert single pattern to array
    const patternArray = Array.isArray(patterns) ? patterns : [patterns];

    // Track statistics
    const stats = {
      processed: 0,
      failed: 0,
      skipped: 0,
    };

    // Process each pattern
    for (const pattern of patternArray) {
      // Find all matching files
      const files = await glob(pattern, { absolute: true });

      // Process each file
      for (const filePath of files) {
        try {
          console.log(`Processing: ${filePath}`);

          // Read file content
          const content = await fs.readFile(filePath, "utf8");

          // Clean the code
          const cleanedContent = cleanProductionCode(content);

          // Skip writing if no changes were made
          if (content.trim() === cleanedContent.trim()) {
            console.log(`No changes needed for: ${filePath}`);
            stats.skipped++;
            continue;
          }

          // Create backup
          const backupPath = `${filePath}.bak`;
          await fs.writeFile(backupPath, content);

          // Write cleaned content
          await fs.writeFile(filePath, cleanedContent);

          console.log(`Successfully processed: ${filePath}`);
          stats.processed++;
        } catch (error) {
          console.error(`Failed to process ${filePath}: ${error.message}`);
          stats.failed++;
        }
      }
    }

    // Print summary
    console.log("\nProcessing Summary:");
    console.log(`Files processed: ${stats.processed}`);
    console.log(`Files skipped (no changes): ${stats.skipped}`);
    console.log(`Files failed: ${stats.failed}`);

    return stats;
  } catch (error) {
    throw new Error(`Error processing files: ${error.message}`);
  }
}

// Command-line interface
async function main() {
  try {
    // Get patterns from command line arguments
    const patterns = process.argv.slice(2);

    if (patterns.length === 0) {
      console.error("Error: No file patterns provided");
      console.log('Usage: node script.js "pattern1" "pattern2" ...');
      console.log('Example: node script.js "**/src/*.rs" "work/*/*.rs"');
      process.exit(1);
    }

    console.log("Processing files with patterns:", patterns);
    await processFiles(patterns);
  } catch (error) {
    console.error("Error:", error.message);
    process.exit(1);
  }
}

// Run if called directly
if (require.main === module) {
  main();
}

export { cleanProductionCode, processFiles };
