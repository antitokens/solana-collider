import { processFiles } from './code-cleaner.js';

// Single pattern
await processFiles('*work/**/this/*.rs');

// Multiple patterns
await processFiles(['*work/**/this/*.rs', 'src/**/*.rs']);