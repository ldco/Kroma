# /pm-apply — Apply Contribution to PM Framework

Applies a contribution document to the PM framework.

## Usage

```
/pm-apply                   # Apply .pm-contribution.md to framework
/pm-apply {contribution_file} # Apply specific contribution file
```

## Algorithm

### 1. Locate Contribution Document

Look for contribution document:
- Default: `.pm-contribution.md`
- Or user-specified file
- Error if not found

### 2. Parse Contribution Document

Extract information from the contribution:
- Summary
- Changes to make
- Files to modify/create
- Implementation details
- Testing instructions

### 3. Apply Changes Safely

For each file in the contribution:
- If file doesn't exist, create it
- If file exists, backup before modifying
- Apply the changes as specified
- Validate syntax/format after changes

### 4. Verify Changes

- Check that all files were created/modified as expected
- Run basic validation (syntax checks)
- Optionally run tests if available

### 5. Generate Confirmation

Display:
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                         ✅ CONTRIBUTION APPLIED
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Applied contribution from: {contribution_file}

Changes made:
- {count} files modified
- {count} files created
- {count} files deleted (if any)

Files affected:
- {file_list}

Verification:
- Syntax validation: {PASS/FAIL}
- Framework integrity: {PASS/FAIL}

Next steps:
1. Review the changes
2. Test the framework functionality
3. Commit the changes if successful

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

## Safety Measures

- Always backup files before modification
- Validate syntax after changes
- Check framework integrity
- Allow rollback if something goes wrong

## Notes

- Contributions should improve the PM framework
- Verify changes work as expected
- Test functionality after applying
- Keep contributions atomic and focused

## Related Commands

| Command | Purpose |
|---------|---------|
| `/pm-contribute` | Create contribution document |