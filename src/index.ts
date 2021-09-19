import { Chart, LinearScale, CategoryScale, LineElement, PointElement, LineController } from 'chart.js'
import { invoke } from '@tauri-apps/api/tauri'
import { emit, Event, EventCallback, listen } from '@tauri-apps/api/event'
import { event } from '@tauri-apps/api';

Chart.register(
    LinearScale,
    CategoryScale,
    LineElement,
    PointElement,
    LineController
);

let chart: Chart;

class ScanResult {
    value: number = 0;
    timestamp: number = 0;
}

class ScanEvent {
    code = "";
    description = "";
}

// Handles / caches
let statusCircleElement: HTMLElement;
let statusTextElement: HTMLElement;
let selectIntervalElement: HTMLSelectElement;
let scanIntervalId = 0;
let index = 1;
// Consts
const STATUS_ATTACHED = "Attached";
const STATUS_DISCONNETED = "Not Attached";

window.addEventListener('DOMContentLoaded', async () => {
    // Get element caches
    let el = document.getElementById("status-circle");
    if (el) { statusCircleElement = el; }
    el = document.getElementById("status-text");
    if (el) { statusTextElement = el; }
    el = document.getElementById("scan-frequency-select");
    if (el) { selectIntervalElement = el as HTMLSelectElement; }

    // Handlers
    document.getElementById('settings-button')?.addEventListener('click', function () {
        let div = document.getElementById('settings-modal');
        if (div) {
            if (div.classList.contains("animate-fade")) {
                div.style.visibility = "visible";
                div.classList.remove("animate-fade");
            } else {
                div.classList.add("animate-fade");
                setTimeout(() => {
                    div!.style.visibility = "hidden";
                }, 200);
            }
        }
    });
    document.getElementById('settings-button')?.addEventListener('mouseenter', function (this) {
        this.classList.toggle('md-inactive');
    });
    document.getElementById('settings-button')?.addEventListener('mouseleave', function (this) {
        this.classList.toggle('md-inactive');
    });
    selectIntervalElement.addEventListener('change', function (this) {
        window.clearInterval(scanIntervalId);
        window.setInterval(get_gil, this.options[this.selectedIndex].value as unknown as number);
    });

    // Initial attach check
    invoke('is_attached').then(function (val) {
        if (val)
            set_attached(true);
    });

    

    // Backend event listeners
    await listen("ScanEvent", event => {
        let payload: ScanEvent = event.payload as ScanEvent;
        console.log(payload.description);
        if (payload.code == "GameConnected")
            set_attached(true);
        else
            set_attached(false);
    });

    var ctx = (<HTMLCanvasElement>document.getElementById('chart')).getContext('2d');
    if (ctx) {
        chart = new Chart(ctx, {
            type: 'line',
            data: {
                labels: [],
                datasets: [{
                    label: '# Of Currency',
                    data: [],
                    backgroundColor: 'rgba(255, 99, 132, 0.2)',
                    borderColor: 'rgba(255, 99, 132, 1)',
                    borderWidth: 1
                }]
            },
            options: {
                plugins: {
                    legend: {
                        display: false
                    }
                },
                maintainAspectRatio: false
            },
        });
    }

    // Load data from disk
    invoke('load_from_disk').then(function(val) {
        let a = val as Array<ScanResult>;
        a.forEach(result => {
            chart.data.labels?.push(index);
            chart.data.datasets.forEach((dataset) => {
                dataset.data.push(result.value);
            });
            index++;
            chart.update();
        });
    });
})

function get_gil() {
    let p = invoke('get_gil');
    p.then((g) => {
        // Wallet value will read as 0 until player is fully logged in
        if (g != 0) {
            chart.data.labels?.push(1);
            chart.data.datasets.forEach((dataset) => {
                dataset.data.push(g as number);
            });
            chart.update();
        }
    }).catch((err) => {
        console.log(err);
    });
}

// Control behavior based on whether the game is attached
function set_attached(attached: boolean) {
    if(attached) {
        statusCircleElement.style.fill = "limegreen";
        statusTextElement.innerHTML = STATUS_ATTACHED;
        scanIntervalId = window.setInterval(function () {
            invoke('get_gil').then(function (value) {
                console.log(value);
            }).catch(function (_) { /* Scan failed for some reason, gobble it */ });
        }, +selectIntervalElement.options[selectIntervalElement.selectedIndex].value);
    } else {
        statusCircleElement.style.fill = "red";
        statusTextElement.innerHTML = STATUS_DISCONNETED;
        window.clearInterval(scanIntervalId);
    }
}