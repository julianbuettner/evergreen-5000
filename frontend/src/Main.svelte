<script>
        import Plant from './Plant.svelte';

        export let waterClock = "09:00h";
        export let lastWateringError = null;
        export let lastConnect = 0;
        let lastConnectDate = new Date(lastConnect);
</script>

<div style="display: flex; flex-direction: row; justify-content: space-evenly;">
        <h1 style="color: white; font-family: comic;">Evergreen 5000</h1>
</div>
<div style="display: flex; flex-direction: row; justify-content: space-evenly;">
        <h2 style="color: white; font-family: comic;">
                Watering daily at {waterClock}
                <br>
                {#if lastWateringError === null }
                Last watering was okay.
                {:else}
                {lastWateringError}
                {/if}
        </h2>
        <h2 style="color: white; font-family: comic;">
                Last contact:<br>
                {lastConnectDate.toISOString()}
        </h2>
</div>


<div style="display: flex; flex-direction: row; flex-wrap: wrap; justify-content: space-evenly;">
        <Plant id=5 name="Bazel" amountMl="500"/>
        <Plant id=5 name="Bazel" amountMl="500"/>
        <Plant id=5 name="Bazel" amountMl="500"/>
        <Plant id=5 name="Bazel" amountMl="500"/>
        <Plant id=5 name="Bazel" amountMl="500"/>
        <Plant id=5 name="Bazel" amountMl="500"/>
</div>

<style>
@font-face {
        font-family: comic;
        src: url("/coolvetica rg.otf");
}
</style>
