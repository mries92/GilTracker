import { Chart } from '../node_modules/chart.js/dist/chart.js'

window.addEventListener('DOMContentLoaded', () => {
    var ctx = document.getElementById('chart').getContext('2d');
    var chart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20],
            datasets: [{
                label: '# Of Currency',
                data: [4,2,3,5,7,12,15,14,13,18,19,22,27,29,31,34,38,43,49,48],
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
            maintainAspectRatio: false
        }
    });
})