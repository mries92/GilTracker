import { Chart } from '../node_modules/chart.js/dist/chart.js'
let chart = Chart;
let currentGil = 20;
let chartIndex = 0;

window.addEventListener('DOMContentLoaded', () => {
    setInterval(addMockData, 5000);
    var ctx = (<HTMLCanvasElement>document.getElementById('chart')).getContext('2d');
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
            scales: {
                y: {
                    beginAtZero: false
                }
            },
            maintainAspectRatio: false,
            normalized: true
        }
    });

    // Generate initial mock data
    let tempData = [];
    let tempLabels = [];
    for (chartIndex = 0; chartIndex < 100; chartIndex++) {
        currentGil += Math.floor(Math.random() * 10) - Math.floor(Math.random() * 8);
        tempLabels.push(chartIndex);
        tempData.push(currentGil);
    }
    chart.data.labels = tempLabels;
    chart.data.datasets[0].data = tempData;
    chart.update();
})

// Add mock data to chart
function addMockData() {
    currentGil += Math.floor(Math.random() * 10) - Math.floor(Math.random() * 8)
    chart.data.labels.push(chartIndex);
    chart.data.datasets[0].data.push(currentGil);
    chartIndex += 1;
    chart.update();
    window.resizeBy(1, 1);
}