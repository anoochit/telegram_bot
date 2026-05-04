const { execSync } = require('child_process');
const path = require('path');
const fs = require('fs');

const args = process.argv.slice(2);
if (args.length < 1) {
    console.error('Usage: node generate_ebook.cjs <input_directory_or_file> [output_name]');
    process.exit(1);
}

const inputPath = path.resolve(args[0]);
const outputBase = args[1] ? args[1] : 'ebook-output';
const pdfOutput = `${outputBase}.pdf`;
const epubOutput = `${outputBase}.epub`;

async function run() {
    try {
        let finalMdPath = inputPath;

        // 1. If it's a directory, merge files
        if (fs.lstatSync(inputPath).isDirectory()) {
            console.log(`Merging files from ${inputPath}...`);
            const files = fs.readdirSync(inputPath)
                .filter(f => f.endsWith('.md'))
                .sort(); // Sort alphabetically or by specific logic if needed

            let mergedContent = '';
            for (const file of files) {
                const filePath = path.join(inputPath, file);
                let content = fs.readFileSync(filePath, 'utf8');
                if (content.charCodeAt(0) === 0xFEFF) content = content.slice(1);
                
                mergedContent += content + '\n\n<div style="page-break-after: always;"></div>\n\n';
            }
            
            finalMdPath = path.join(process.cwd(), '.temp-merged-book.md');
            fs.writeFileSync(finalMdPath, mergedContent, 'utf8');
        }

        // 2. Generate PDF
        console.log('Generating PDF...');
        const pdfCmd = `node "${path.join(__dirname, '../../create-pdf/scripts/generate_pdf.cjs')}" "${finalMdPath}" "${pdfOutput}"`;
        execSync(pdfCmd, { stdio: 'inherit' });

        // 3. Generate EPUB
        console.log('Generating EPUB...');
        const epubCmd = `node "${path.join(__dirname, '../../create-epub/scripts/generate_epub.cjs')}" "${finalMdPath}" "${epubOutput}"`;
        execSync(epubCmd, { stdio: 'inherit' });

        // Cleanup temp file if created
        if (finalMdPath.includes('.temp-merged-book.md') && fs.existsSync(finalMdPath)) {
            fs.unlinkSync(finalMdPath);
        }

        console.log('\n--- Success! ---');
        console.log(`PDF: ${pdfOutput}`);
        console.log(`EPUB: ${epubOutput}`);

    } catch (error) {
        console.error('\n--- Error ---');
        console.error(error.message);
        process.exit(1);
    }
}

run();
