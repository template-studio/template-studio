import * as vscode from 'vscode';

/**
 * 变量装饰器 - 在编辑器中高亮显示模板变量
 */
export class VariableDecorationProvider implements vscode.Disposable {
    private decorationType: vscode.TextEditorDecorationType;
    private disposables: vscode.Disposable[] = [];

    constructor() {
        // 变量装饰样式
        this.decorationType = vscode.window.createTextEditorDecorationType({
            backgroundColor: 'rgba(66, 133, 244, 0.1)',
            border: '1px solid rgba(66, 133, 244, 0.3)',
            borderRadius: '3px',
            cursor: 'pointer',
        });

        // 监听文档变化
        this.disposables.push(
            vscode.workspace.onDidChangeTextDocument(event => {
                const editor = vscode.window.visibleTextEditors.find(
                    e => e.document.uri === event.document.uri
                );
                if (editor) {
                    this.updateDecorations(editor);
                }
            })
        );

        // 初始化当前编辑器
        if (vscode.window.activeTextEditor) {
            this.updateDecorations(vscode.window.activeTextEditor);
        }
    }

    /**
     * 更新装饰器
     */
    updateDecorations(editor: vscode.TextEditor) {
        const document = editor.document;
        const text = document.getText();

        // 匹配 {{ variable }} 和 {{ variable | filter }}
        const regex = /\{\{\s*([\w]+)(?:\s*\|[^}]*)?\s*\}\}/g;
        const decorations: vscode.DecorationOptions[] = [];

        let match;
        while ((match = regex.exec(text))) {
            const startPos = document.positionAt(match.index);
            const endPos = document.positionAt(match.index + match[0].length);
            const range = new vscode.Range(startPos, endPos);

            decorations.push({
                range,
                hoverMessage: `变量: **${match[1]}**\n\n点击可切换显示原始值/渲染值`,
            });
        }

        editor.setDecorations(this.decorationType, decorations);
    }

    dispose() {
        this.decorationType.dispose();
        this.disposables.forEach(d => d.dispose());
    }
}
