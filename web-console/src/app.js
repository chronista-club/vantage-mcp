document.addEventListener('alpine:init', () => {
  Alpine.data('app', () => ({
    processes: [],
    showCreateModal: false,
    newProcess: {
      id: '',
      command: '',
      args: '',
      cwd: '',
      auto_start_on_restore: false,
      env: {}
    },

    async init() {
      await this.loadProcesses();
      // 5秒ごとに自動更新
      setInterval(() => this.loadProcesses(), 5000);
    },

    async loadProcesses() {
      try {
        const response = await fetch('/api/processes');
        if (response.ok) {
          this.processes = await response.json();
        }
      } catch (error) {
        console.error('Failed to load processes:', error);
      }
    },

    async createProcess() {
      try {
        const args = this.newProcess.args ? this.newProcess.args.split(' ').filter(a => a) : [];
        const payload = {
          ...this.newProcess,
          args,
          env: {}
        };

        const response = await fetch('/api/processes', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(payload)
        });

        if (response.ok) {
          this.showCreateModal = false;
          this.resetNewProcess();
          await this.loadProcesses();
        } else {
          const error = await response.text();
          alert('Failed to create process: ' + error);
        }
      } catch (error) {
        console.error('Failed to create process:', error);
        alert('Failed to create process');
      }
    },

    async startProcess(id) {
      try {
        const response = await fetch(`/api/processes/${id}/start`, {
          method: 'POST'
        });

        if (response.ok) {
          await this.loadProcesses();
        } else {
          const error = await response.text();
          alert('Failed to start process: ' + error);
        }
      } catch (error) {
        console.error('Failed to start process:', error);
      }
    },

    async stopProcess(id) {
      try {
        const response = await fetch(`/api/processes/${id}/stop`, {
          method: 'POST'
        });

        if (response.ok) {
          await this.loadProcesses();
        } else {
          const error = await response.text();
          alert('Failed to stop process: ' + error);
        }
      } catch (error) {
        console.error('Failed to stop process:', error);
      }
    },

    async removeProcess(id) {
      if (!confirm(`Are you sure you want to remove process ${id}?`)) {
        return;
      }

      try {
        const response = await fetch(`/api/processes/${id}`, {
          method: 'DELETE'
        });

        if (response.ok) {
          await this.loadProcesses();
        } else {
          const error = await response.text();
          alert('Failed to remove process: ' + error);
        }
      } catch (error) {
        console.error('Failed to remove process:', error);
      }
    },

    resetNewProcess() {
      this.newProcess = {
        id: '',
        command: '',
        args: '',
        cwd: '',
        auto_start_on_restore: false,
        env: {}
      };
    }
  }));
});