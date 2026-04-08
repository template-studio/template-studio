import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

/**
 * CLI 命令执行器
 * 通过子进程调用 template-studio CLI
 */
export class CliExecutor {
    private cliPath: string;

    constructor(cliPath?: string) {
        this.cliPath = cliPath || 'ts';
    }

    /**
     * 执行 CLI 命令并解析 JSON 输出
     */
    private async execute(args: string[]): Promise<any> {
        const command = `${this.cliPath} ${args.join(' ')}`;
        try {
            const { stdout, stderr } = await execAsync(command, {
                timeout: 60000,
                maxBuffer: 10 * 1024 * 1024,
            });

            if (stderr) {
                console.warn('CLI stderr:', stderr);
            }

            return JSON.parse(stdout.trim());
        } catch (err: any) {
            if (err.stdout) {
                try {
                    const errorOutput = JSON.parse(err.stdout.trim());
                    throw new Error(errorOutput.error || errorOutput.message || '未知错误');
                } catch {
                    // stdout 不是 JSON
                }
            }
            throw new Error(err.message || 'CLI 执行失败');
        }
    }

    /**
     * 分析模板变量
     */
    async analyzeVariables(templatePath: string): Promise<any> {
        return this.execute(['ai', 'analyze-variables', templatePath, '--format', 'json']);
    }

    /**
     * AI 填充变量
     */
    async fillVariables(templatePath: string, projectId: number): Promise<any> {
        return this.execute([
            'ai', 'fill-variables', templatePath,
            '--project', projectId.toString(),
            '--format', 'json'
        ]);
    }

    /**
     * 渲染预览
     */
    async renderPreview(
        templatePath: string,
        varsFile?: string,
        full?: boolean
    ): Promise<any> {
        const args = ['ai', 'render-preview', templatePath];
        if (varsFile) {
            args.push('--vars-file', varsFile);
        }
        if (full) {
            args.push('--full');
        }
        return this.execute(args);
    }

    /**
     * 语法验证
     */
    async validateSyntax(templatePath: string): Promise<any> {
        return this.execute(['ai', 'validate', templatePath]);
    }

    /**
     * 转换项目为模板
     */
    async convertToTemplate(
        projectPath: string,
        outputPath: string,
        name?: string,
        category?: string,
        strategy?: string
    ): Promise<any> {
        const args = ['ai', 'convert-to-template', projectPath, '--output', outputPath];
        if (name) args.push('--name', name);
        if (category) args.push('--category', category);
        if (strategy) args.push('--strategy', strategy);
        return this.execute(args);
    }

    /**
     * 文件编辑
     */
    async editFile(
        filePath: string,
        operation: string,
        line?: number,
        content?: string
    ): Promise<any> {
        const args = ['ai', 'edit-file', filePath, `--${operation}`];
        if (line !== undefined) {
            args.push(line.toString());
        }
        if (content) {
            args.push('--content', content);
        }
        return this.execute(args);
    }

    /**
     * 模板推荐
     */
    async recommend(
        language?: string,
        category?: string,
        explain?: boolean
    ): Promise<any> {
        const args = ['ai', 'recommend'];
        if (language) args.push('--language', language);
        if (category) args.push('--category', category);
        if (explain) args.push('--explain');
        return this.execute(args);
    }
}
