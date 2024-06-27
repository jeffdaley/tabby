'use client'

import React from 'react'
import Link from 'next/link'
import LazyLoad from 'react-lazy-load'

import { filename2prism } from '@/lib/language-utils'
import { IconFile } from '@/components/ui/icons'

import CodeEditorView from './code-editor-view'
import { SourceCodeBrowserContext } from './source-code-browser'
import { SourceCodeSearchResult as SourceCodeSearchResultType } from './source-code-search-results'
import { generateEntryPath } from './utils'

interface SourceCodeSearchResultProps {
  result: SourceCodeSearchResultType
  query: string
}

export const SourceCodeSearchResult = ({
  ...props
}: SourceCodeSearchResultProps) => {
  const { activeRepo, activeRepoRef } = React.useContext(
    SourceCodeBrowserContext
  )

  const language = filename2prism(props.result.path)[0]

  const [ranges, setRanges] = React.useState<{ start: number; end: number }[]>(
    []
  )

  const [firstLineWithSubMatch, setFirstLineWithSubMatch] = React.useState<
    number | null
  >(null)

  React.useEffect(() => {
    const newRanges: { start: number; end: number }[] = []
    let currentRange: { start: number; end: number } = { start: 0, end: 0 }

    props.result.lines.forEach((line, index) => {
      if (line.subMatches.length > 0) {
        setFirstLineWithSubMatch(line.lineNumber)
      }
      if (index === 0) {
        currentRange.start = line.lineNumber
        currentRange.end = line.lineNumber
      } else {
        if (line.lineNumber === currentRange.end + 1) {
          currentRange.end = line.lineNumber
        } else {
          newRanges.push(currentRange)
          currentRange = { start: line.lineNumber, end: line.lineNumber }
        }
      }
    })

    newRanges.push(currentRange)

    setRanges(newRanges)
  }, [props.result.lines])

  const pathname = `/files/${generateEntryPath(
    activeRepo,
    activeRepoRef?.name as string,
    props.result.path,
    'file'
  )}`

  return (
    <div>
      <div className="sticky top-0 bg-background z-10">
        <Link
          href={{
            pathname,
            hash: `L${firstLineWithSubMatch}`
          }}
          className="mb-2 font-medium inline-flex text-primary hover:underline"
        >
          {props.result.path}
        </Link>
      </div>
      <div className="grid border divide-y divide-y-border border-border">
        {ranges.map((range, i) => (
          <LazyLoad key={`${props.result.path}-${i}`}>
            <Link
              href={{
                pathname,
                // Account for the contextual lines provided by the backend
                hash: `L${range.start + 3}`
              }}
              className="group relative"
            >
              <div className="absolute left-0 w-full h-full top-0 hidden group-hover:block group-focus:block bg-accent"></div>
              <div className="group-hover:opacity-75 group-focus:opacity-75">
                <CodeEditorView
                  value={props.result.blob}
                  language={language}
                  stringToMatch={props.query}
                  lineRange={range}
                  interactionsAreDisabled={true}
                />
              </div>
            </Link>
          </LazyLoad>
        ))}
      </div>
    </div>
  )
}
