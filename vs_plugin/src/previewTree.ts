import * as vscode from 'vscode';
import { CliExecutor } from './executor';

/**
 * 渲染预览 TreeView 提供器
 */
export class PreviewTreeProvider implements vscode.TreeDataProvider<PreviewItem> {
    private _onDidChangeTreeData = new vscode.EventEmitter<PreviewItem | undefined | void>();
    readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

    private executor: CliExecutor;
    private currentUri: vscode.Uri | undefined;

    constructor(executor: CliExecutor) {
        this.executor = executor;
    }

    refresh(uri?: vscode.Uri): void {
        this.currentUri = uri;
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: PreviewItem): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: PreviewItem): Promise<PreviewItem[]> {
        if (element) {
            return [];
        }

        if (!this.currentUri) {
            return [];
        }

        try {
            const result = await this.executor.renderPreview(this.currentUri.fsPath, undefined, false);
            const files = result.files || [];

            return files.map((f: any) => new PreviewItem(
                f.path,
                f.success,
                f.size || 0,
                f.error
            ));
        } catch {
            return [];
        }
    }
}

class PreviewItem extends vscode.TreeItem {
    constructor(
        public readonly filePath: string,
        public readonly success: boolean,
        public readonly size: number,
        public readonly error?: string
    ) {
        super(filePath, vscode.TreeItemCollapsibleState.None);

        this.description = success
            ? `${(size / 1024).toFixed(1)}KB`
            : '渲染失败';

        this.tooltip = success
            ? `${filePath} (${size} bytes)`
            : `${filePath}: ${error}`;

        this.contextValue = 'preview-file';

        this.iconPath = new vscode.ThemeIcon(
            success ? 'file' : 'error',
            success ? undefined : new vscode.ThemeColor('errorForeground')
        );
    }
}
