#!/usr/bin/env node

/**
 * API Envelope Audit Script
 * 
 * Validates that all API endpoints return the strict success envelope:
 * { success: true, data: ... }
 * 
 * Excluded Paths (Intentional Exceptions per Release 1.3.0):
 * 1. /api/ws - WebSocket protocol handler (not HTTP response)
 * 2. /api/docs/openapi.json - OpenAPI spec document (static JSON, not API response)
 * 3. /api/docs/swagger - Swagger UI HTML page (static HTML, not API response)
 * 4. /api/media/s3/* - S3 proxy stream handler (returns binary stream, not JSON)
 * 
 * These endpoints are protocol/documentation endpoints, not standard API routes.
 * They do not return JSON responses and are correctly excluded from envelope validation.
 * 
 * @see docs/architecture/1.3.0.md - Task 2.1: API Envelope Documentation
 */

import fs from 'node:fs'
import path from 'node:path'
import ts from 'typescript'

const projectRoot = process.cwd()
const apiRoot = path.join(projectRoot, 'server', 'api')
const enforce = process.argv.includes('--enforce')

/**
 * Excluded path patterns - these are protocol/documentation endpoints
 * that do not return standard JSON API responses.
 * 
 * Per CONTRIBUTING.md Section 14 (Audit Backlog):
 * - 138/141 routes use strict envelope
 * - 3 intentional exceptions: ws, OpenAPI JSON, Swagger HTML
 */
const EXCLUDED_PATH_PATTERNS = [
  // WebSocket protocol handler - returns WS connection, not HTTP response
  /[\\/]server[\\/]api[\\/]ws\.ts$/,
  // OpenAPI specification document - static JSON file, not API response
  /[\\/]server[\\/]api[\\/]docs[\\/]openapi\.json\.get\.ts$/,
  // Swagger UI - returns HTML page, not JSON API response
  /[\\/]server[\\/]api[\\/]docs[\\/]swagger\.get\.ts$/,
  // S3 media proxy - returns binary stream via sendStream(), not JSON
  /[\\/]server[\\/]api[\\/]media[\\/]s3[\\/]/
]

function collectApiFiles(dir) {
  const entries = fs.readdirSync(dir, { withFileTypes: true })
  const files = []

  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name)
    if (entry.isDirectory()) {
      files.push(...collectApiFiles(fullPath))
      continue
    }

    if (entry.isFile() && entry.name.endsWith('.ts')) {
      files.push(fullPath)
    }
  }

  return files
}

function isExcluded(filePath) {
  return EXCLUDED_PATH_PATTERNS.some(pattern => pattern.test(filePath))
}

function isNamedProperty(property, name) {
  if (!('name' in property) || !property.name) {
    return false
  }

  if (ts.isIdentifier(property.name)) {
    return property.name.text === name
  }

  if (ts.isStringLiteral(property.name)) {
    return property.name.text === name
  }

  return false
}

function unwrapExpression(expression) {
  if (ts.isParenthesizedExpression(expression)) {
    return unwrapExpression(expression.expression)
  }

  if (ts.isAsExpression(expression)) {
    return unwrapExpression(expression.expression)
  }

  if (ts.isSatisfiesExpression(expression)) {
    return unwrapExpression(expression.expression)
  }

  return expression
}

function findHandlerFunction(sourceFile) {
  for (const statement of sourceFile.statements) {
    if (!ts.isExportAssignment(statement)) {
      continue
    }

    const expression = unwrapExpression(statement.expression)
    if (!ts.isCallExpression(expression)) {
      continue
    }

    const callee = unwrapExpression(expression.expression)
    if (!ts.isIdentifier(callee) || callee.text !== 'defineEventHandler') {
      continue
    }

    const firstArg = expression.arguments[0]
    if (
      firstArg &&
      (ts.isArrowFunction(firstArg) || ts.isFunctionExpression(firstArg))
    ) {
      return firstArg
    }
  }

  return null
}

function classifyReturnedObject(objectExpression) {
  const successProp = objectExpression.properties.find(
    property =>
      ts.isPropertyAssignment(property) &&
      isNamedProperty(property, 'success') &&
      unwrapExpression(property.initializer).kind === ts.SyntaxKind.TrueKeyword
  )

  const dataProp = objectExpression.properties.find(
    property =>
      (ts.isPropertyAssignment(property) || ts.isShorthandPropertyAssignment(property)) &&
      isNamedProperty(property, 'data')
  )

  if (successProp && dataProp) return 'compliant'
  if (successProp) return 'legacy_success_shape'
  return 'no_envelope'
}

function classifyEnvelopeStatus(source, filePath) {
  const sourceFile = ts.createSourceFile(filePath, source, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS)
  const handler = findHandlerFunction(sourceFile)

  if (!handler?.body || !ts.isBlock(handler.body)) {
    return 'unknown'
  }

  const returnClassifications = []

  function visit(node) {
    if (node !== handler && (ts.isArrowFunction(node) || ts.isFunctionExpression(node) || ts.isFunctionDeclaration(node))) {
      return
    }

    if (ts.isReturnStatement(node) && node.expression) {
      const returnedExpression = unwrapExpression(node.expression)
      if (ts.isObjectLiteralExpression(returnedExpression)) {
        returnClassifications.push(classifyReturnedObject(returnedExpression))
      } else {
        returnClassifications.push('unknown')
      }
      return
    }

    ts.forEachChild(node, visit)
  }

  visit(handler.body)

  if (returnClassifications.length === 0) {
    return 'unknown'
  }

  if (returnClassifications.every(status => status === 'compliant')) {
    return 'compliant'
  }

  if (returnClassifications.some(status => status === 'legacy_success_shape')) {
    return 'legacy_success_shape'
  }

  if (returnClassifications.some(status => status === 'no_envelope')) {
    return 'no_envelope'
  }

  return 'unknown'
}

if (!fs.existsSync(apiRoot)) {
  console.error(`API directory not found: ${apiRoot}`)
  process.exit(1)
}

const allApiFiles = collectApiFiles(apiRoot)
const analyzedFiles = allApiFiles.filter(file => !isExcluded(file))

const results = analyzedFiles.map(file => {
  const source = fs.readFileSync(file, 'utf8')
  return {
    file: path.relative(projectRoot, file),
    status: classifyEnvelopeStatus(source, file)
  }
})

const compliant = results.filter(item => item.status === 'compliant')
const nonCompliant = results.filter(item => item.status !== 'compliant')

console.log('API Envelope Audit')
console.log(`- total analyzed: ${results.length}`)
console.log(`- compliant: ${compliant.length}`)
console.log(`- non-compliant: ${nonCompliant.length}`)
console.log(`- excluded: ${allApiFiles.length - analyzedFiles.length}`)

if (nonCompliant.length > 0) {
  console.log('\nNon-compliant handlers:')
  for (const item of nonCompliant) {
    console.log(`- [${item.status}] ${item.file}`)
  }
}

if (enforce && nonCompliant.length > 0) {
  process.exit(1)
}
