<script>
        import { onMount } from 'svelte';
        import Plant from './Plant.svelte';

        export let waterClock = "09:00h";
        let plants = [];
        let lastSeenInfo = {
                lastSeenTimestamp: 0,
                lastBatteryPercentage: 0.0,
                lastWateringDate: '',
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
                const fromUnix = new Date(ts * 1000);
                const dateString = Intl.DateTimeFormat('de-de', {dateStyle: 'medium'}).format(fromUnix);
                const timeString = Intl.DateTimeFormat('de-de', {timeStyle: 'medium'}).format(fromUnix);
                return timeString + ", " + dateString;
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
        <h3 class="info-header">
                Watering daily<br>
                {waterClock}
        </h3>
        {#await lastSeenInfo}
                <h3 class="info-header">Infos are loading...</h3>
        {:then lastSeen}
        <h3 class="info-header" style="text-align: center">
                Last contact<br>
                {formatTimestamp(lastSeen.lastSeenTimestamp)}
        </h3>
        <h3 class="info-header" style="text-align: center">
                Last watering<br>
                {lastSeen.lastWateringDate}
        </h3>
        <h3 class="info-header" style="text-align: right">
                Battery<br>
                {lastSeen.lastBatteryPercentage}%
        </h3>
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
}

.info-header-box {
        display: flex;
        flex-direction: row;
        justify-content: space-evenly;
        /* border: 3px solid white; */
        border-radius: 8px;
        margin: 1%;
        margin-left: 8%;
        margin-right: 8%;
}
</style>
