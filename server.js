const express = require("express");
const cors = require("cors");
const axios = require("axios");

const app = express();
app.use(cors());
app.use(express.json()); // Ensure JSON body is parsed correctly

// Proxy "/update" to Rust server
app.post("/update", async (req, res) => {
    try {
        console.log("Forwarding request:", req.body); // Debugging output

        const rustResponse = await axios.post(
            "http://127.0.0.1:3000/update",
            req.body, // Ensure request body is passed correctly
            {
                headers: { "Content-Type": "application/json" }, // Use correct header
            }
        );
        console.log(rustResponse.data)
        res.json(rustResponse.data);
    } catch (error) {
        console.error("Error forwarding request to Rust:", error.response?.data || error.message);
        res.status(error.response?.status || 500).json({
            error: "Failed to connect to Rust server",
            details: error.response?.data || error.message,
        });
    }
});

// Start Express server on port 4000
app.listen(4000, () => {
    console.log("Node.js server running on http://127.0.0.1:4000");
});
