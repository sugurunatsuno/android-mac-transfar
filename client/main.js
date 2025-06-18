const form = document.getElementById('upload-form');
const fileInput = document.getElementById('file-input');
const progress = document.getElementById('progress');
const info = document.getElementById('info');
const dirForm = document.getElementById('dir-form');
const dirInput = document.getElementById('dir-input');

const entries = {};

const events = new EventSource('/events');
events.onmessage = (e) => {
  const data = JSON.parse(e.data);
  let el = entries[data.file];
  if (!el) {
    el = document.createElement('div');
    el.textContent = `${data.file}: starting`;
    progress.appendChild(el);
    entries[data.file] = el;
  }
  if (data.status === 'progress') {
    el.textContent = `${data.file}: ${data.bytes} bytes`;
  } else if (data.status === 'done') {
    el.textContent = `${data.file}: complete`;
  }
};

form.addEventListener('submit', async (e) => {
  e.preventDefault();
  const fd = new FormData();
  for (const file of fileInput.files) {
    fd.append('file', file, file.name);
  }
  await fetch('/upload', { method: 'POST', body: fd });
});

async function loadInfo() {
  const res = await fetch('/info');
  if (!res.ok) return;
  const data = await res.json();
  info.textContent = `IPs: ${data.ips.join(', ')} Port: ${data.port}`;
  dirInput.value = data.dir;
}

dirForm.addEventListener('submit', async (e) => {
  e.preventDefault();
  await fetch('/set_dir', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ dir: dirInput.value }) });
  loadInfo();
});

loadInfo();
