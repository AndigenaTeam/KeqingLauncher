import TextInputPart from "./TextInputPart.tsx";

export default function TextInput({ name, value, placeholder, readOnly}: { name: string, value: string, placeholder?: string, readOnly: boolean }) {

    return (
        <div className="flex flex-row items-center justify-between w-full h-8">
            <span className="text-white text-sm">{name}</span>
            <div className={"overflow-ellipsis inline-flex flex-row items-center justify-center"}>
                <TextInputPart initalValue={value} placeholder={placeholder} readOnly={readOnly} isPicker={false}/>
            </div>
        </div>
    )
}
