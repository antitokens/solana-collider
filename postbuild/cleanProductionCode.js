import { glob } from "glob";
import fs from "fs/promises";

// Core cleaning function remains the same but without any formatting logic
function cleanProductionCode(sourceCode) {
  try {
    let lines = sourceCode.split("\n");
    let cleanedLines = [];

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

      // Skip lines with "Remove in production" comment
      if (currentLine.includes("// CRITICAL: Remove in production")) {
        continue;
      }

      // Handle multi-line blocks ending with "Remove in production"
      if (currentLine.includes("// CRITICAL: Remove in production!")) {
        let j = cleanedLines.length - 1;
        let openBraces = 0;
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
