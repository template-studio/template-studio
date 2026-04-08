import * as vscode from 'vscode';
import { CliExecutor } from './executor';
import { VariableDecorationProvider } from './decorator';
import { VariableTreeProvider } from './variableTree';
import { PreviewTreeProvider } from './previewTree';
import { ValidationTreeProvider } from './validateTree';

export function activate(context: vscode.ExtensionContext) {
    const executor = new CliExecutor();

    // 注册命令
    context.subscriptions.push(
        vscode.commands.registerCommand('templateStudio.convertToTemplate', async () => {
            await convertToTemplate(executor);
        }),
        vscode.commands.registerCommand('templateStudio.analyzeVariables', async () => {
            await analyzeVariables(executor);
        }),
        vscode.commands.registerCommand('templateStudio.renderPreview', async () => {
            await renderPreview(executor);
        }),
        vscode.commands.registerCommand('templateStudio.validate', async () => {
            await validate(executor);
        }),
        vscode.commands.registerCommand('templateStudio.fillVariables', async () => {
            await fillVariables(executor);
        }),
        vscode.commands.registerCommand('templateStudio.uploadTemplate', async () => {
            await uploadTemplate(executor);
        }),
        vscode.commands.registerCommand('templateStudio.validateResults.refresh', () => {
            validationTreeProvider.refresh(vscode.window.activeTextEditor?.document.uri);
        })
    );

    // 注册变量装饰器
    const decorator = new VariableDecorationProvider();
    context.subscriptions.push(decorator);

    // 注册 TreeView
    const variableTreeProvider = new VariableTreeProvider(executor);
    const previewTreeProvider = new PreviewTreeProvider(executor);
    const validationTreeProvider = new ValidationTreeProvider(executor);

    vscode.window.registerTreeDataProvider('templateStudio.variables', variableTreeProvider);
    vscode.window.registerTreeDataProvider('templateStudio.preview', previewTreeProvider);
    vscode.window.registerTreeDataProvider('templateStudio.validation', validationTreeProvider);

    // 监听编辑器变化
    context.subscriptions.push(
        vscode.window.onDidChangeActiveTextEditor(editor => {
            if (editor) {
                decorator.updateDecorations(editor);
                variableTreeProvider.refresh(editor.document.uri);
                validationTreeProvider.refresh(editor.document.uri);
            }
        })
    );

    console.log('Template Studio 扩展已激活');
}

async function convertToTemplate(executor: CliExecutor) {
    const projectPath = await vscode.window.showInputBox({
        prompt: '输入项目路径',
        placeHolder: '/path/to/project',
        value: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath
    });
    if (!projectPath) return;

    const outputPath = await vscode.window.showInputBox({
        prompt: '输入输出模板路径',
        placeHolder: '/path/to/output/template'
    });
    if (!outputPath) return;

    const name = await vscode.window.showInputBox({
        prompt: '模板名称（可选）',
        placeHolder: 'My Template'
    });

    const category = await vscode.window.showInputBox({
        prompt: '模板分类（可选）',
        placeHolder: 'web, cli, library'
    });

    const strategy = await vscode.window.showQuickPick(
        ['conservative', 'aggressive'],
        { placeHolder: '变量识别策略' }
    );

    await vscode.window.withProgress({
        location: vscode.ProgressLocation.Notification,
        title: '正在转换项目为模板...',
        cancellable: false
    }, async (progress) => {
        try {
            const result = await executor.convertToTemplate(
                projectPath,
                outputPath,
                name,
                category,
                strategy || 'conservative'
            );

            vscode.window.showInformationMessage(
                `转换完成！创建了 ${result.files} 个模板文件，${result.variables} 个变量`
            );

            // 打开输出目录
            const uri = vscode.Uri.file(outputPath);
            vscode.commands.executeCommand('vscode.openFolder', uri, true);
        } catch (err: any) {
            vscode.window.showErrorMessage(`转换失败: ${err.message}`);
        }
    });
}

async function analyzeVariables(executor: CliExecutor) {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showWarningMessage('请先打开一个模板文件');
        return;
    }

    const filePath = editor.document.uri.fsPath;

    try {
        const result = await executor.analyzeVariables(filePath);

        // 在输出面板显示结果
        const outputChannel = vscode.window.createOutputChannel('Template Studio');
        outputChannel.clear();
        outputChannel.appendLine('=== 变量分析结果 ===');
        outputChannel.appendLine(`模板: ${result.template}`);
        outputChannel.appendLine(`变量数量: ${result.total}`);
        outputChannel.appendLine('');
        for (const v of result.variables) {
            outputChannel.appendLine(`  ${v.name} (${v.type})`);
            if (v.description) {
                outputChannel.appendLine(`    ${v.description}`);
            }
        }
        outputChannel.show();

        // 刷新变量面板
        vscode.commands.executeCommand('templateStudio.variables.refresh');
    } catch (err: any) {
        vscode.window.showErrorMessage(`分析失败: ${err.message}`);
    }
}

async function renderPreview(executor: CliExecutor) {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showWarningMessage('请先打开一个模板文件');
        return;
    }

    const filePath = editor.document.uri.fsPath;

    try {
        const result = await executor.renderPreview(filePath, undefined, true);

        const outputChannel = vscode.window.createOutputChannel('Template Studio');
        outputChannel.clear();
        outputChannel.appendLine('=== 渲染预览 ===');
        outputChannel.appendLine(`总文件数: ${result.total}`);
        outputChannel.appendLine(`成功: ${result.success}`);
        outputChannel.appendLine(`失败: ${result.failed}`);
        outputChannel.appendLine('');
        for (const file of result.files) {
            const status = file.success ? '✓' : '✗';
            outputChannel.appendLine(`${status} ${file.path}`);
            if (file.content) {
                outputChannel.appendLine('---');
                outputChannel.appendLine(file.content.substring(0, 500));
                if (file.content.length > 500) {
                    outputChannel.appendLine('... (truncated)');
                }
                outputChannel.appendLine('---');
            }
            if (file.error) {
                outputChannel.appendLine(`  错误: ${file.error}`);
            }
        }
        outputChannel.show();
    } catch (err: any) {
        vscode.window.showErrorMessage(`渲染失败: ${err.message}`);
    }
}

async function validate(executor: CliExecutor) {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showWarningMessage('请先打开一个模板文件');
        return;
    }

    const filePath = editor.document.uri.fsPath;

    try {
        const result = await executor.validateSyntax(filePath);

        if (result.valid) {
            vscode.window.showInformationMessage('模板语法验证通过');
        } else {
            const outputChannel = vscode.window.createOutputChannel('Template Studio');
            outputChannel.clear();
            outputChannel.appendLine('=== 验证错误 ===');
            for (const error of result.errors) {
                outputChannel.appendLine(`  ✗ ${error}`);
            }
            outputChannel.show();
        }
    } catch (err: any) {
        vscode.window.showErrorMessage(`验证失败: ${err.message}`);
    }
}

async function fillVariables(executor: CliExecutor) {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showWarningMessage('请先打开一个模板文件');
        return;
    }

    const filePath = editor.document.uri.fsPath;

    const projectIdStr = await vscode.window.showInputBox({
        prompt: '输入项目 ID',
        placeHolder: '1'
    });
    if (!projectIdStr) return;

    const projectId = parseInt(projectIdStr);
    if (isNaN(projectId)) {
        vscode.window.showErrorMessage('无效的项目 ID');
        return;
    }

    await vscode.window.withProgress({
        location: vscode.ProgressLocation.Notification,
        title: 'AI 正在填充变量...',
        cancellable: false
    }, async () => {
        try {
            const result = await executor.fillVariables(filePath, projectId);

            const outputChannel = vscode.window.createOutputChannel('Template Studio');
            outputChannel.clear();
            outputChannel.appendLine('=== AI 变量填充结果 ===');
            outputChannel.appendLine(`模板: ${result.template}`);
            outputChannel.appendLine(`置信度: ${(result.confidence * 100).toFixed(1)}%`);
            outputChannel.appendLine(`推理: ${result.ai_reasoning}`);
            outputChannel.appendLine('');
            outputChannel.appendLine('填充的变量:');
            outputChannel.appendLine(JSON.stringify(result.filled, null, 2));
            outputChannel.show();

            vscode.window.showInformationMessage(
                `AI 已填充变量，置信度: ${(result.confidence * 100).toFixed(1)}%`
            );
        } catch (err: any) {
            vscode.window.showErrorMessage(`填充失败: ${err.message}`);
        }
    });
}

async function uploadTemplate(executor: CliExecutor) {
    const templatePath = await vscode.window.showInputBox({
        prompt: '输入模板目录路径',
        placeHolder: '/path/to/template',
        value: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath
    });
    if (!templatePath) return;

    const serverUrl = await vscode.window.showInputBox({
        prompt: '输入服务器 URL（可选，使用默认配置）',
        placeHolder: 'http://localhost:8080'
    });

    const apiKey = await vscode.window.showInputBox({
        prompt: '输入 API Key（可选，使用默认配置）',
        password: true
    });

    await vscode.window.withProgress({
        location: vscode.ProgressLocation.Notification,
        title: '正在上传模板...',
        cancellable: false
    }, async (progress) => {
        try {
            // 先验证模板
            progress.report({ message: '验证模板语法...' });
            const validationResult = await executor.validateSyntax(templatePath);
            if (!validationResult.valid) {
                const choice = await vscode.window.showWarningMessage(
                    `模板有 ${validationResult.errors.length} 个错误，是否继续上传？`,
                    '继续上传',
                    '取消'
                );
                if (choice !== '继续上传') return;
            }

            // 构建上传命令参数
            const args = ['template', 'upload', templatePath];
            if (serverUrl) args.push('--server', serverUrl);
            if (apiKey) args.push('--api-key', apiKey);

            progress.report({ message: '上传中...' });

            // 执行上传（通过 CLI）
            const { stdout, stderr } = await require('util').promisify(require('child_process').exec)(
                `${executor['cliPath']} ${args.join(' ')}`,
                { timeout: 120000 }
            );

            if (stderr) {
                console.warn('Upload stderr:', stderr);
            }

            vscode.window.showInformationMessage('模板上传成功！');

            // 刷新验证面板
            vscode.commands.executeCommand('templateStudio.validateResults.refresh');
        } catch (err: any) {
            vscode.window.showErrorMessage(`上传失败: ${err.message}`);
        }
    });
}

export function deactivate() {}
