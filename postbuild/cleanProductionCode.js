import { glob } from "glob";
import fs from "fs/promises";

// Core cleaning function remains the same but with robust logic for block removal
function cleanProductionCode(sourceCode) {
  try {
    let lines = sourceCode.split("\n");
    let cleanedLines = [];
    let skipMode = false;
    let openBraces = 0;

    for (let i = 0; i < lines.length; i++) {
      const currentLine = lines[i];

      // Handle "Add in production" comments
      if (currentLine.includes("// CRITICAL: Add in production!")) {
        const contentToAdd = currentLine.split("!")[1];
        if (contentToAdd) {
          cleanedLines.push(contentToAdd.trim());
        }
        continue;
      }

      // Handle "Remove in production" comment and start block skip mode
      if (currentLine.includes("// CRITICAL: Remove in production!")) {
        skipMode = true;
        openBraces = 0;
        continue;
      }

      if (skipMode) {
        // Track braces to handle block removal
        openBraces += (currentLine.match(/{/g) || []).length;
        openBraces -= (currentLine.match(/}/g) || []).length;

        // End skip mode if we've closed the block
        if (openBraces <= 0) {
          skipMode = false;
        }
        continue;
      }

      // Add the line if it wasn't skipped
      cleanedLines.push(currentLine);
    }

    return cleanedLines.join("\n");
  } catch (error) {
    throw new Error(`Error cleaning code: ${error.message}`);
  }
}

// Function to process multiple files
async function processFiles(patterns) {
  try {
    const patternArray = Array.isArray(patterns) ? patterns : [patterns];

    const stats = {
      processed: 0,
      failed: 0,
      skipped: 0,
    };

    for (const pattern of patternArray) {
      const files = await glob(pattern, { absolute: true });

      for (const filePath of files) {
        try {
          console.log(`Processing: ${filePath}`);
          const content = await fs.readFile(filePath, "utf8");

          const cleanedContent = cleanProductionCode(content);

          if (content.trim() === cleanedContent.trim()) {
            console.log(`No changes needed for: ${filePath}`);
            stats.skipped++;
            continue;
          }

          const backupPath = `${filePath}.bak`;
          await fs.writeFile(backupPath, content);
          await fs.writeFile(filePath, cleanedContent);

          console.log(`Successfully processed: ${filePath}`);
          stats.processed++;
        } catch (error) {
          console.error(`Failed to process ${filePath}: ${error.message}`);
          stats.failed++;
        }
      }
    }

    console.log("\nProcessing Summary:");
    console.log(`Files processed: ${stats.processed}`);
    console.log(`Files skipped (no changes): ${stats.skipped}`);
    console.log(`Files failed: ${stats.failed}`);

    return stats;
  } catch (error) {
    throw new Error(`Error processing files: ${error.message}`);
  }
}

export { cleanProductionCode, processFiles };
