# /pm-contribute — Export Fix/Feature as Contribution

Exports a fix or feature from a client project as a contribution document for the PM framework.

## Usage

```
/pm-contribute                # Export current changes as contribution
/pm-contribute {feature}      # Export specific feature as contribution
```

## Algorithm

### 1. Identify Changes to Export

Detect what has been modified/added:
- Git diff to identify changed files
- Or user specifies specific feature/files
- Focus on reusable improvements to PM framework

### 2. Create Contribution Document

Generate `.pm-contribution.md` with:

```
# PM Framework Contribution

## Summary
Brief description of the contribution and its purpose.

## Changes Made
- File: `path/to/file` - Description of changes
- File: `path/to/file` - Description of changes

## Files Included
\`\`\`
path/to/file1
path/to/file2
\`\`\`

## Implementation Details
Detailed explanation of the changes and why they were made.

## Impact
Description of how this improves the PM framework.

## Testing
Steps taken to verify the changes work correctly.

## Notes
Any additional information for the framework maintainers.
```

### 3. Extract Relevant Code

Include the actual code changes in the document:
- For new files: full content
- For modified files: diff or new version
- Focus on PM framework improvements, not client-specific code

### 4. Generate Output

Display:
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                         ✅ CONTRIBUTION CREATED
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Contribution saved to: .pm-contribution.md

Contents:
{display content of .pm-contribution.md}

To apply this contribution to the PM framework:
1. Copy .pm-contribution.md to the PM framework repo
2. Run /pm-apply to implement the changes

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

## Notes

- Focus on framework improvements, not client-specific features
- Include enough context for framework maintainers to understand
- Test changes before contributing
- Keep contributions focused and atomic

## Related Commands

| Command | Purpose |
|---------|---------|
| `/pm-apply` | Apply contribution to PM framework |