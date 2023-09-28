<script>
        export let name;
        export let amountMl;
        export let allowWateringTest = true;
        let pendingWatering = null;

        const updateAmount = async () => {
                console.log("New amount: " + amountMl);
                const requestRes = await fetch(
                        "/api/updateml/" + name + "?amountMl=" + amountMl,
                        {
                                method: "POST"
                        }
                );
                const body = await requestRes.text();
                console.log("Result of setting amountMl: " + body);
        }

        const startTestWatering = async () => {
                console.log('Start watering test. This will block until it is fulfilled.');
                const requestRes = await fetch(
                        "/api/testwatering/" + name,
                        {method: "POST"}
                );
                if (requestRes.status == 410) {
                        return 410;
                }
                if (requestRes.status == 403) {
                        return 403;
                }
                const body = await requestRes.text();
                console.log("Result of watering test: " + body);
                return body;
        }
</script>

<div style="padding: 4px; font-family: comic;">
<div style="border: 6px solid green; width: 270px; padding: 10px; border-radius: 12px;">
         <div style="display: flex; justify-content: center; align-items: center; padding: 8px;">
                <img
                        src="/plant.png" alt="Plant"
                        width=200
                        height=200
                        style="object-fit: cover"
                >

         </div>
         <div style="display: flex; justify-content: center; align-items: center; padding: 8px; color: white; font-size: 29px;">
                {name}
         </div>
         <div style="display: flex; justify-content: center; align-items: center; padding: 4px; color: white">
         <select name="fml2" id="selectAmountMl" bind:value={amountMl} on:change={updateAmount}>
                <option value="{amountMl}" selected>{amountMl}ml / day</option>
                {#each [0, 10, 25, 50, 100, 250, 500] as amountMlOption }
                <option value="{amountMlOption}">{amountMlOption}ml / day</option>
                {/each}
         </select>
         </div>
         {#if allowWateringTest}
         {#await pendingWatering}
                 <div class="info-text">
                        Waiting... <br>
                        <hr>
                        The controller usually sleeps for multiple minutes.
                        Restart it to wake it up and run the watering test.
                        To abort the test leave or refresh this page.
                 </div>
         {:then doneWatering}
                 <div style="display: flex; justify-content: center; align-items: center; padding: 4px; color: white">
                        <button
                                style="font-family: comic; text-decoriation: none; color: #2b2b2b; background-color: white; border: none; padding: 5px 17px; border-radius: 4px;"
                                on:click={() => pendingWatering = startTestWatering()}
                        >
                                Test watering
                        </button>
                 </div>

                 {#if doneWatering == null}
                 <!-- Do nothing -->
                 {:else if doneWatering == 410}
                 <p class="info-text">
                        Another watering test has been started,
                        so this one has been canceled.
                 </p>
                 {:else if doneWatering == 403}
                 <p class="info-text">
                        You have to be in the same WLAN
                        as the evergreen 5000 to start watering tests.
                 </p>
                 {:else}
                 <p class="info-text">
                        Water test will start now!<br>
                 </p>
                 {/if}
         {:catch error}
         {/await}
         {/if}
 </div>
 </div>

 <style>
.info-text {
        font-family: comic;
        color: white;
        align: center;
}
 </style>
