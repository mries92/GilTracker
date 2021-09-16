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
//let currentGil = 20;
//let chartIndex = 0;

window.addEventListener('DOMContentLoaded', () => {
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

    setInterval(function(){
        let p = invoke('get_gil');
        p.then((g) => {
            // Wallet value will read as 0 until player is fully logged in
            if(g != 0) {
                chart.data.labels?.push(1);
                chart.data.datasets.forEach((dataset) => {
                    dataset.data.push(g as number);
                });
                chart.update();
            }
        }).catch(() => {
            console.log("Error reading gil from game.");
        });
    }, 60000 * 3); // A sane default for now I think, reads a value every 3 minutes

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