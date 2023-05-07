
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


<style>
    field {
        font-weight: 140;
    }

    date {
        font-size: 1rem;
        font-weight: 150;
        word-wrap: break-all;
        color: #cccccc;
    }

    facet {
        text-align: right;
        font-size: 0.8rem;
        font-weight: 150;
        color: #cccccc;
    }
</style>