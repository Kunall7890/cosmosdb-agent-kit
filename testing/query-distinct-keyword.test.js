

// testing/query-distinct-keyword.test.js



const fs = require('fs');

const path = require('path');



describe('query-distinct-keyword rule', () => {

  it('should exist', () => {

    const filePath = path.join(__dirname, '../skills/cosmosdb-best-practices/rules/query-distinct-keyword.md');

    const exists = fs.existsSync(filePath);

    expect(exists).toBe(true);

  });



  it('should have frontmatter with title, impact, and tags', () => {

    const filePath = path.join(__dirname, '../skills/cosmosdb-best-practices/rules/query-distinct-keyword.md');

    const content = fs.readFileSync(filePath, 'utf-8');

    expect(content).toMatch(/title:\s*Use DISTINCT keyword/);

    expect(content).toMatch(/impact:\s*MEDIUM/);

    expect(content).toMatch(/tags:/);

  });



  it('should contain correct and incorrect examples', () => {

    const filePath = path.join(__dirname, '../skills/cosmosdb-best-practices/rules/query-distinct-keyword.md');

    const content = fs.readFileSync(filePath, 'utf-8');

    expect(content).toMatch(/Incorrect \(client-side deduplication\)/);

    expect(content).toMatch(/Correct \(using DISTINCT in Cosmos DB\)/);

    expect(content).toMatch(/Correct \(using DISTINCT VALUE for scalar results\)/);

  });

});