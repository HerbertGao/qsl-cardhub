import MarkdownIt from 'markdown-it'
import DOMPurify from 'dompurify'

const md = new MarkdownIt({
  html: false,
  linkify: true,
  breaks: true,
})

const PURIFY_CONFIG = {
  ALLOWED_TAGS: [
    'h1', 'h2', 'h3', 'h4',
    'p', 'br', 'hr',
    'ul', 'ol', 'li',
    'a',
    'strong', 'b', 'em', 'i', 'del', 's',
    'code', 'pre',
    'blockquote',
  ],
  ALLOWED_ATTR: ['href', 'title', 'target', 'rel'],
}

/**
 * 将 Markdown 源文本渲染为安全的 HTML 字符串。
 * 使用 markdown-it 渲染后经 DOMPurify 清洗，防止 XSS。
 */
export function renderMarkdown(source: string): string {
  const rawHtml = md.render(source)
  return DOMPurify.sanitize(rawHtml, PURIFY_CONFIG) as string
}
