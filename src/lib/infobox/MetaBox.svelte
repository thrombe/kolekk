
<script lang="ts">
    import type { Meta, TypeFacet } from "types";

    export let item: Meta<unknown, TypeFacet>;
</script>


<facet>
    <field>Type: </field>
    {item.facet}
</facet>
<date>
    <field>Created On: </field>
    {new Date(Number(item.ctime) * 1000).toLocaleDateString(
        'en-US',
        {
            month: 'long',
            day: 'numeric',
            year: 'numeric'
        }
    )}
</date>
<date>
    <field>Last Modified: </field>
    {new Date(
        Number(item.last_update) * 1000
    ).toLocaleDateString('en-US', {
        month: 'long',
        day: 'numeric',
        year: 'numeric'
    })}
</date>


<style lang='postcss'>
    field {
        @apply break-all text-gray-400 font-thin;
    }

    date {
        @apply break-all font-thin text-base text-gray-300;
    }

    facet {
        @apply break-all font-thin text-sm text-gray-300 text-right;
    }
</style>
