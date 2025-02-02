import prettier from 'prettier';

function cleanProductionCode(sourceCode) {
    // Split the code into lines for processing
    let lines = sourceCode.split('\n');
    let cleanedLines = [];
    let skipNextLines = false;
    
    for (let i = 0; i < lines.length; i++) {
        const currentLine = lines[i];
        
        // Case 3: Handle "Add in production" comments
        if (currentLine.includes('// CRITICAL: Add in production!')) {
            const contentToAdd = currentLine.split('!')[1];
            if (contentToAdd) {
                cleanedLines.push(contentToAdd.trim());
            }
            continue;
        }
        
        // Case 1: Skip lines with "Remove in production" comment
        if (currentLine.includes('// CRITICAL: Remove in production')) {
            continue;
        }
        
        // Case 2: Check for multi-line blocks ending with "Remove in production"
        if (currentLine.includes('// CRITICAL: Remove in production!')) {
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
    let cleanedCode = cleanedLines.join('\n');
    
    // Format the code using prettier
    try {
        cleanedCode = prettier.format(cleanedCode, {
            parser: 'typescript',
            semi: true,
            singleQuote: true,
            tabWidth: 4,
        });
    } catch (error) {
        console.warn('Warning: Prettier formatting failed, returning unformatted code', error);
    }
    
    return cleanedCode;
}

// Test the function
const testCode = `
    struct MyStruct {
        name: String,
        unix_timestamp: Option<i64>, // CRITICAL: Remove in production
        value: i32,
    }

    let now = match unix_timestamp {
        Some(ts) => ts,
        None => Clock::get()?.unix_timestamp,
    }; // CRITICAL: Remove in production!

    // CRITICAL: Add in production!let timestamp = Clock::get()?.unix_timestamp;

    fn process_data() {
        let x = 5;
        // Normal comment
        let y = 10;
    }
`;

console.log('Original code:');
console.log(testCode);
console.log('\nCleaned code:');
console.log(cleanProductionCode(testCode));

export default cleanProductionCode;
