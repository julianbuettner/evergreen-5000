<script>
        import { onMount } from 'svelte';
        import Plant from './Plant.svelte';

        export let waterClock = "09:00h";
        let plants = [];
        let lastSeenInfo = {
                lastSeenTimestamp: 0,
                lastBatteryPercentage: 0.0,
        }

        async function getPlants() {
                const response = await fetch('/api/plants');
                return await response.json();
        }
        async function getLastSeen() {
                const response = await fetch('/api/lastseen');
                return await response.json();
        };
        function formatTimestamp(ts) {
                const fromUnix = new Date(ts);
                const month = fromUnix.getMonth() + 1;
                const day = fromUnix.getDate();
                const hours = fromUnix.getHours();
                const minutes = fromUnix.getMinutes();
                const seconds = fromUnix.getSeconds();
                return hours + ":" + minutes + ":" + seconds + "h, " + day + "." + month;
        }
        onMount(() => {
                plants = getPlants();
                lastSeenInfo = getLastSeen();
        })
</script>

<div style="display: flex; flex-direction: row; justify-content: space-evenly;">
        <h1 style="color: white; font-family: comic;">Evergreen 5000</h1>
</div>
<div 
        class="info-header-box"
>
        <h2 class="info-header">
                Watering daily<br>
                {waterClock}
        </h2>
        {#await lastSeenInfo}
                <h2 class="info-header">Infos are loading...</h2>
        {:then lastSeen}
        <h2 class="info-header">
                Last contact<br>
                {formatTimestamp(lastSeen.lastSeenTimestamp)}
        </h2>
        <h2 class="info-header">
                Battery<br>
                {lastSeen.lastBatteryPercentage}%
        </h2>
        {:catch error}
                <p>Something went wrong: {error.message}
        {/await}
</div>


<div style="display: flex; flex-direction: row; flex-wrap: wrap; justify-content: space-evenly;">
        {#await plants}
                <p>Plants are loading...</p>
        {:then plantConfigs}
                {#each plantConfigs as plantConfig}
                <Plant {...plantConfig}/>
                {/each}
        {:catch error}
                <p>Something went wrong: {error.message}
        {/await}
</div>

<style>
@font-face {
        font-family: comic;
        src: url("/coolvetica rg.otf");
}

.info-header {
        color: white;
        font-family: comic;
        text-align: center;
}

.info-header-box {
        display: flex;
        flex-direction: row;
        justify-content: space-evenly;
        border: 3px solid white;
        border-radius: 12px;
        margin: 1%;
        margin-left: 10%;
        margin-right: 10%;
}
</style>
