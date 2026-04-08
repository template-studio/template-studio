import * as vscode from 'vscode';
import { CliExecutor } from './executor';

/**
 * 变量 TreeView 提供器
 */
export class VariableTreeProvider implements vscode.TreeDataProvider<VariableItem> {
    private _onDidChangeTreeData = new vscode.EventEmitter<VariableItem | undefined | void>();
    readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

    private executor: CliExecutor;
    private currentUri: vscode.Uri | undefined;
    private variables: VariableInfo[] = [];

    constructor(executor: CliExecutor) {
        this.executor = executor;
    }

    refresh(uri?: vscode.Uri): void {
        this.currentUri = uri;
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: VariableItem): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: VariableItem): Promise<VariableItem[]> {
        if (element) {
            return [];
        }

        if (!this.currentUri) {
            return [];
        }

        // 分析变量
        try {
            const result = await this.executor.analyzeVariables(this.currentUri.fsPath);
            this.variables = result.variables || [];

            return this.variables.map(v => new VariableItem(
                v.name,
                v.type || 'string',
                v.description || '',
                v.required ?? true
            ));
        } catch {
            return [];
        }
    }
}

interface VariableInfo {
    name: string;
    type: string;
    description: string;
    required: boolean;
}

class VariableItem extends vscode.TreeItem {
    constructor(
        public readonly name: string,
        public readonly type: string,
        public readonly description: string,
        public readonly required: boolean
    ) {
        super(name, vscode.TreeItemCollapsibleState.None);

        this.description = `${type}${required ? ' (必填)' : ''}`;
        this.tooltip = description || name;

        this.contextValue = 'variable';

        this.iconPath = new vscode.ThemeIcon(
            required ? 'variable' : 'symbol-variable',
            required ? undefined : new vscode.ThemeColor('disabledForeground')
        );

        this.command = {
            command: 'editor.action.goToLocations',
            title: '跳转到变量',
            arguments: [this.resourceUri, new vscode.Position(0, 0), []]
        };
    }
}
