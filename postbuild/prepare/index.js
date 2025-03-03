import { glob } from "glob";
import fs from "fs/promises";
import { process } from "./clean.js";

async function replaceCargoTestFilesContent(pattern) {
    try {
        const files = await glob(pattern, { absolute: true });

        const newTestContent = `#[tokio::test]
async fn test_full_collider_flow() {
    println!("\\nNote: No testing available for production branch. Please switch to 'localnet' branch for testing");
}
        `;

        let processed = 0;
        let failed = 0;

        for (const filePath of files) {
            if (filePath.endsWith(".rs")) {
                try {
                    console.log(`Replacing content of: ${filePath}`);
                    await fs.writeFile(filePath, newTestContent);
                    processed++;
                } catch (error) {
                    console.error(`Failed to process ${filePath}: ${error.message}`);
                    failed++;
                }
            }
        }

        console.log("\nCargo Replacement Summary:");
        console.log(`Files successfully processed: ${processed}`);
        console.log(`Files failed: ${failed}`);

    } catch (error) {
        console.error(`Error replacing test file content: ${error.message}`);
    }
}

async function replaceAnchorTestFilesContent(pattern) {
    try {
        const files = await glob(pattern, { absolute: true });

        const newTestContent = `console.log("\\nNote: No testing available for production branch. Please switch to 'localnet' branch for testing");`;

        let processed = 0;
        let failed = 0;

        for (const filePath of files) {
            if (filePath.endsWith(".ts")) {
                try {
                    console.log(`Replacing content of: ${filePath}`);
                    await fs.writeFile(filePath, newTestContent);
                    processed++;
                } catch (error) {
                    console.error(`Failed to process ${filePath}: ${error.message}`);
                    failed++;
                }
            }
        }

        console.log("\nAnchor Replacement Summary:");
        console.log(`Files successfully processed: ${processed}`);
        console.log(`Files failed: ${failed}`);

    } catch (error) {
        console.error(`Error replacing test file content: ${error.message}`);
    }
}

// Cleanup .bak files function
async function cleanupBackupFiles(pattern) {
    try {
        console.log("\nCleanup Summary:");
        const backupFiles = await glob(pattern, { absolute: true });
        for (const backupFile of backupFiles) {
            try {
                await fs.unlink(backupFile);
                console.log(`Removed backup file: ${backupFile}`);
            } catch (error) {
                console.error(`Failed to remove backup file ${backupFile}: ${error.message}`);
            }
        }
    } catch (error) {
        console.error(`Error cleaning up backup files: ${error.message}`);
    }
}

async function main() {
    // Process test files with the specific pattern
    await process(["programs/collider-beta/src/**/*.rs"]);

    // Replace test file content for all Cargo test files
    await replaceCargoTestFilesContent("programs/collider-beta/tests/*.rs");

    // Replace test file content for all Anchor test files
    await replaceAnchorTestFilesContent("tests/*.ts");

    // Clean backup files
    await cleanupBackupFiles("programs/collider-beta/src/**/*.bak");
}

main().catch(console.error);

