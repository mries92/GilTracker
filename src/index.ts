import { Chart, LinearScale, CategoryScale, LineElement, PointElement, LineController } from 'chart.js'
import { invoke } from '@tauri-apps/api/tauri'

Chart.register(
    LinearScale,
    CategoryScale,
    LineElement,
    PointElement,
    LineController
);

let chart: Chart;
let data: number[] = [];

// Handles / caches
let statusCircleElement: HTMLElement;
let statusTextElement: HTMLElement;
let scanIntervalId = 0;
// Consts
const STATUS_ATTACHED = "Attached";
const STATUS_DISCONNETED = "Not Attached";

window.addEventListener('DOMContentLoaded', () => {
    // Get element caches
    let el = document.getElementById("status-circle");
    if(el) { statusCircleElement = el; }
    el = document.getElementById("status-text");
    if(el) { statusTextElement = el; }

    // Handlers
    document.getElementById('settings-button')?.addEventListener('click', function() {
        let div = document.getElementById('settings-modal');
        if(div) {
            if(div.classList.contains("animate-fade")) {
                div.style.visibility = "visible";
                div.classList.remove("animate-fade");
            } else {
                div.classList.add("animate-fade");
                setTimeout( () => {
                    div!.style.visibility = "hidden";
                }, 200);
            }
        }
    });
    document.getElementById('settings-button')?.addEventListener('mouseenter', function(this) {
        this.classList.toggle('md-inactive');
    });
    document.getElementById('settings-button')?.addEventListener('mouseleave', function(this) {
        this.classList.toggle('md-inactive');
    });
    document.getElementById('scan-frequency-select')?.addEventListener('change', function(this: HTMLSelectElement) {
        window.clearInterval(scanIntervalId);
        window.setInterval(get_gil, this.options[this.selectedIndex].value as unknown as number);
    });

    var ctx = (<HTMLCanvasElement>document.getElementById('chart')).getContext('2d');
    if(ctx) {
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

    // Kick off an initial scan when booting up
    get_gil();
})

function get_gil() {
    console.log("DEBUG: In get gil JS.");
    let p = invoke('get_gil');
    p.then((g) => {
        statusCircleElement.style.fill = "limegreen";
        statusTextElement.innerHTML = STATUS_ATTACHED;
        // Wallet value will read as 0 until player is fully logged in
        if(g != 0) {
            chart.data.labels?.push(1);
            chart.data.datasets.forEach((dataset) => {
                dataset.data.push(g as number);
            });
            chart.update();
            data.push(g as number);
        }
    }).catch((err) => {
        statusCircleElement.style.fill = "red";
        statusTextElement.innerHTML = STATUS_DISCONNETED;
        console.log(err);
    });
}