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
let currentGil = 20;
let chartIndex = 0;

window.addEventListener('DOMContentLoaded', () => {
    document.getElementById('test-button')?.addEventListener('click', function() {
        addMockData();
    });
    setInterval(scanForGil, 10000);
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
})

function scanForGil() {
    invoke('');
}

// Add mock data to chart
function addMockData() {
    currentGil += Math.floor(Math.random() * 10) - Math.floor(Math.random() * 8)
    chart.data.labels?.push(chartIndex);
    if(chart.data.datasets)
        chart.data.datasets[0].data?.push(currentGil);
    chartIndex += 1;
    chart.update();
    window.resizeBy(1, 1);
}