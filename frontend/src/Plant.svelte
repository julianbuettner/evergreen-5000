<script>
        export let id;
        export let name;
        export let amountMl;
        export let allowWateringTest = true;
</script>

<div style="padding: 4px; font-family: comic;">
<div style="border: 6px solid green; width: 270px; padding: 10px; border-radius: 12px;">
         <div style="display: flex; justify-content: center; align-items: center; padding: 8px;">
                <img
                        src="/plant.png" alt="Image of plant"
                        width=200
                        height=200
                        style="object-fit: cover"
                >

         </div>
         <div style="display: flex; justify-content: center; align-items: center; padding: 8px; color: white; font-size: 29px;">
                {name}
         </div>
         <div style="display: flex; justify-content: center; align-items: center; padding: 4px; color: white">
         {amountMl}ml / day
         </div>
         {#if allowWateringTest}
         <div style="display: flex; justify-content: center; align-items: center; padding: 4px; color: white">
                <button style="font-family: comic; text-decoriation: none; color: #2b2b2b; background-color: white; border: none; padding: 5px 17px; border-radius: 4px;">
                Test watering
                </button>
         </div>
         {/if}
 </div>
 </div>
