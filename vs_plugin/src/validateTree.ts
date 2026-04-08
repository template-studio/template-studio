import * as vscode from 'vscode';
import { CliExecutor } from './executor';

/**
 * 验证结果 TreeView 提供器
 */
export class ValidationTreeProvider implements vscode.TreeDataProvider<ValidationItem> {
    private _onDidChangeTreeData = new vscode.EventEmitter<ValidationItem | undefined | void>();
    readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

    private executor: CliExecutor;
    private currentUri: vscode.Uri | undefined;
    private results: ValidationResult[] = [];

    constructor(executor: CliExecutor) {
        this.executor = executor;
    }

    refresh(uri?: vscode.Uri): void {
        this.currentUri = uri;
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: ValidationItem): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: ValidationItem): Promise<ValidationItem[]> {
        if (element) {
            // 展开错误详情
            return element.errors.map(err => new ValidationItem(
                err,
                'error-detail',
                [],
                element.filePath
            ));
        }

        if (!this.currentUri) {
            return [];
        }

        try {
            const syntaxResult = await this.executor.validateSyntax(this.currentUri.fsPath);

            if (syntaxResult.valid) {
                return [new ValidationItem(
                    '验证通过',
                    'success',
                    [],
                    this.currentUri.fsPath
                )];
            }

            return syntaxResult.errors.map((err: string) => new ValidationItem(
                err,
                'error',
                [err],
                this.currentUri.fsPath
            ));
        } catch (err: any) {
            return [new ValidationItem(
                `验证失败: ${err.message}`,
                'error',
                [err.message],
                this.currentUri?.fsPath
            )];
        }
    }
}

interface ValidationResult {
    valid: boolean;
    errors: string[];
}

class ValidationItem extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly status: 'success' | 'error' | 'error-detail',
        public readonly errors: string[],
        public readonly filePath?: string
    ) {
        super(label, errors.length > 0 ? vscode.TreeItemCollapsibleState.Collapsed : vscode.TreeItemCollapsibleState.None);

        this.contextValue = status;

        switch (status) {
            case 'success':
                this.iconPath = new vscode.ThemeIcon('check', new vscode.ThemeColor('testing.iconPassed'));
                this.description = '无错误';
                break;
            case 'error':
                this.iconPath = new vscode.ThemeIcon('error', new vscode.ThemeColor('testing.iconFailed'));
                this.description = `${errors.length} 个错误`;
                if (filePath) {
                    this.command = {
                        command: 'vscode.open',
                        title: '打开文件',
                        arguments: [vscode.Uri.file(filePath)]
                    };
                }
                break;
            case 'error-detail':
                this.iconPath = new vscode.ThemeIcon('circle-filled', new vscode.ThemeColor('testing.iconFailed'));
                this.description = '';
                break;
        }
    }
}
