// Alpine.js Process Manager Application
document.addEventListener('alpine:init', () => {
    Alpine.data('processManager', () => ({
        // Data
        processes: [],
        filteredProcesses: [],
        searchQuery: '',
        showCreateModal: false,
        showLogsModal: false,
        currentProcessId: '',
        currentLogs: '',
        logStream: 'both',
        
        // New process form
        newProcess: {
            id: '',
            command: '',
            argsString: '',
            cwd: '',
            envString: '',
        },
        
        // Computed properties
        get runningCount() {
            return this.processes.filter(p => p.info.state.state === 'Running').length;
        },
        
        get stoppedCount() {
            return this.processes.filter(p => p.info.state.state === 'Stopped').length;
        },
        
        get failedCount() {
            return this.processes.filter(p => p.info.state.state === 'Failed').length;
        },
        
        // Initialize
        init() {
            this.loadProcesses();
            this.filteredProcesses = this.processes;
            
            // Auto-refresh every 5 seconds
            setInterval(() => {
                this.refreshData();
            }, 5000);
        },
        
        // Load processes from API
        async loadProcesses() {
            try {
                const response = await fetch('/api/processes');
                if (response.ok) {
                    this.processes = await response.json();
                    this.filterProcesses();
                }
            } catch (error) {
                console.error('Failed to load processes:', error);
                this.showToast('プロセスの読み込みに失敗しました', 'error');
            }
        },
        
        // Refresh data
        async refreshData() {
            await this.loadProcesses();
        },
        
        // Filter processes based on search query
        filterProcesses() {
            if (!this.searchQuery) {
                this.filteredProcesses = this.processes;
                return;
            }
            
            const query = this.searchQuery.toLowerCase();
            this.filteredProcesses = this.processes.filter(p => {
                return p.info.id.toLowerCase().includes(query) ||
                       p.info.command.toLowerCase().includes(query);
            });
        },
        
        // Create new process
        async createProcess() {
            const args = this.newProcess.argsString 
                ? this.newProcess.argsString.split(' ').filter(a => a) 
                : [];
            
            const env = {};
            if (this.newProcess.envString) {
                this.newProcess.envString.split('\n').forEach(line => {
                    const [key, value] = line.split('=');
                    if (key && value) {
                        env[key.trim()] = value.trim();
                    }
                });
            }
            
            const payload = {
                id: this.newProcess.id,
                command: this.newProcess.command,
                args: args,
                env: env,
                cwd: this.newProcess.cwd || null,
            };
            
            try {
                const response = await fetch('/api/processes', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(payload),
                });
                
                if (response.ok) {
                    this.showCreateModal = false;
                    this.resetNewProcessForm();
                    await this.loadProcesses();
                    this.showToast('プロセスを作成しました', 'success');
                } else {
                    const error = await response.text();
                    this.showToast(`エラー: ${error}`, 'error');
                }
            } catch (error) {
                console.error('Failed to create process:', error);
                this.showToast('プロセスの作成に失敗しました', 'error');
            }
        },
        
        // Start process
        async startProcess(id) {
            try {
                const response = await fetch(`/api/processes/${id}/start`, {
                    method: 'POST',
                });
                
                if (response.ok) {
                    await this.loadProcesses();
                    this.showToast(`プロセス ${id} を開始しました`, 'success');
                } else {
                    const error = await response.text();
                    this.showToast(`エラー: ${error}`, 'error');
                }
            } catch (error) {
                console.error('Failed to start process:', error);
                this.showToast('プロセスの開始に失敗しました', 'error');
            }
        },
        
        // Stop process
        async stopProcess(id) {
            try {
                const response = await fetch(`/api/processes/${id}/stop`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ grace_period_ms: 5000 }),
                });
                
                if (response.ok) {
                    await this.loadProcesses();
                    this.showToast(`プロセス ${id} を停止しました`, 'success');
                } else {
                    const error = await response.text();
                    this.showToast(`エラー: ${error}`, 'error');
                }
            } catch (error) {
                console.error('Failed to stop process:', error);
                this.showToast('プロセスの停止に失敗しました', 'error');
            }
        },
        
        // Delete process
        async deleteProcess(id) {
            if (!confirm(`プロセス ${id} を削除してもよろしいですか？`)) {
                return;
            }
            
            try {
                const response = await fetch(`/api/processes/${id}`, {
                    method: 'DELETE',
                });
                
                if (response.ok) {
                    await this.loadProcesses();
                    this.showToast(`プロセス ${id} を削除しました`, 'success');
                } else {
                    const error = await response.text();
                    this.showToast(`エラー: ${error}`, 'error');
                }
            } catch (error) {
                console.error('Failed to delete process:', error);
                this.showToast('プロセスの削除に失敗しました', 'error');
            }
        },
        
        // View logs
        async viewLogs(id) {
            this.currentProcessId = id;
            this.showLogsModal = true;
            await this.loadLogs();
        },
        
        // Load logs
        async loadLogs() {
            try {
                const response = await fetch(`/api/processes/${this.currentProcessId}/logs?stream=${this.logStream}&lines=100`);
                if (response.ok) {
                    const logs = await response.json();
                    this.currentLogs = logs.join('\n') || 'ログがありません';
                } else {
                    this.currentLogs = 'ログの取得に失敗しました';
                }
            } catch (error) {
                console.error('Failed to load logs:', error);
                this.currentLogs = 'ログの取得に失敗しました';
            }
        },
        
        // Set log stream filter
        async setLogStream(stream) {
            this.logStream = stream;
            await this.loadLogs();
        },
        
        // Reset new process form
        resetNewProcessForm() {
            this.newProcess = {
                id: '',
                command: '',
                argsString: '',
                cwd: '',
                envString: '',
            };
        },
        
        // Get status badge class
        getStatusBadgeClass(state) {
            switch (state.state) {
                case 'Running':
                    return 'badge-running';
                case 'Stopped':
                    return 'badge-stopped';
                case 'Failed':
                    return 'badge-failed';
                case 'NotStarted':
                    return 'badge-notstarted';
                default:
                    return 'badge-secondary';
            }
        },
        
        // Get status text
        getStatusText(state) {
            switch (state.state) {
                case 'Running':
                    return '実行中';
                case 'Stopped':
                    return '停止';
                case 'Failed':
                    return '失敗';
                case 'NotStarted':
                    return '未開始';
                default:
                    return state.state;
            }
        },
        
        // Format date
        formatDate(date) {
            if (!date) return '-';
            const d = new Date(date);
            if (isNaN(d.getTime())) return '-';
            
            const pad = (n) => n.toString().padStart(2, '0');
            return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ` +
                   `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
        },
        
        // Show toast notification
        showToast(message, type = 'info') {
            // Simple console log for now - can be replaced with a proper toast library
            console.log(`[${type.toUpperCase()}] ${message}`);
            
            // You could integrate with Tabler's toast component here
            const toast = document.createElement('div');
            toast.className = `alert alert-${type === 'error' ? 'danger' : type} alert-dismissible position-fixed bottom-0 end-0 m-3`;
            toast.style.zIndex = '9999';
            toast.innerHTML = `
                ${message}
                <button type="button" class="btn-close" data-bs-dismiss="alert"></button>
            `;
            document.body.appendChild(toast);
            
            setTimeout(() => {
                toast.remove();
            }, 3000);
        },
    }));
});